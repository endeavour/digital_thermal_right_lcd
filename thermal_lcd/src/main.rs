use hidapi::HidDevice;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const NUMBER_OF_LEDS: usize = 115;
const VENDOR_ID: u16 = 0x0416;
const PRODUCT_ID: u16 = 0x8001;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    #[serde(rename = "display_mode", default = "default_display_mode")]
    display_mode: String,
    #[serde(rename = "color_mode", default = "default_color_mode")]
    color_mode: String,
    #[serde(rename = "gpu_vendor", default = "default_gpu_vendor")]
    gpu_vendor: String,
    #[serde(rename = "update_interval", default = "default_update_interval")]
    update_interval: f64,
    #[serde(rename = "metrics_update_interval", default = "default_metrics_interval")]
    metrics_update_interval: f64,
    #[serde(rename = "cycle_duration", default = "default_cycle_duration")]
    cycle_duration: f64,
    #[serde(rename = "vendor_id", default = "default_vendor_id")]
    vendor_id: String,
    #[serde(rename = "product_id", default = "default_product_id")]
    product_id: String,
    #[serde(rename = "cpu_temperature_unit", default = "default_temp_unit")]
    cpu_temperature_unit: String,
    #[serde(rename = "gpu_temperature_unit", default = "default_temp_unit")]
    gpu_temperature_unit: String,
    #[serde(default)]
    usage: UsageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UsageConfig {
    #[serde(default)]
    colors: Vec<String>,
}

fn default_display_mode() -> String { "cpu_watts".to_string() }
fn default_color_mode() -> String { "usage".to_string() }
fn default_gpu_vendor() -> String { "nvidia".to_string() }
fn default_update_interval() -> f64 { 0.1 }
fn default_metrics_interval() -> f64 { 1.0 }
fn default_cycle_duration() -> f64 { 5.0 }
fn default_vendor_id() -> String { "0x0416".to_string() }
fn default_product_id() -> String { "0x8001".to_string() }
fn default_temp_unit() -> String { "celsius".to_string() }

impl Default for Config {
    fn default() -> Self {
        Config {
            display_mode: default_display_mode(),
            color_mode: default_color_mode(),
            gpu_vendor: default_gpu_vendor(),
            update_interval: default_update_interval(),
            metrics_update_interval: default_metrics_interval(),
            cycle_duration: default_cycle_duration(),
            vendor_id: default_vendor_id(),
            product_id: default_product_id(),
            cpu_temperature_unit: default_temp_unit(),
            gpu_temperature_unit: default_temp_unit(),
            usage: UsageConfig::default(),
        }
    }
}

struct Layout {
    usage_percent_led: usize,
    usage_1s_digit: Vec<usize>,
    usage_10s_digit: Vec<usize>,
    usage_100s_led: usize,
    speed_mhz_led: usize,
    speed_digits: Vec<Vec<usize>>,
    temp_cpu_led: usize,
    temp_gpu_led: usize,
    temp_100s_digit: Vec<usize>,
    temp_10s_digit: Vec<usize>,
    temp_1s_digit: Vec<usize>,
    temp_fahrenheit: usize,
    temp_celsius: usize,
    watts_w_led: usize,
    watts_digits: Vec<Vec<usize>>,
}

impl Default for Layout {
    fn default() -> Self {
        // Map from segment name (a-g) to LED index
        Layout {
            usage_percent_led: 0,
            usage_1s_digit: vec![6, 5, 1, 2, 3, 7, 4],   // map: a=6,b=5,c=1,d=2,e=3,f=7,g=4
            usage_10s_digit: vec![13, 12, 8, 9, 10, 14, 11], // map: a=13,b=12,c=8,d=9,e=10,f=14,g=11
            usage_100s_led: 15,
            speed_mhz_led: 16,
            speed_digits: vec![
                vec![22, 21, 17, 18, 19, 23, 20],  // digit 1: a=22,b=21,c=17,d=18,e=19,f=23,g=20
                vec![29, 28, 24, 25, 26, 30, 27],  // digit 2
                vec![36, 35, 31, 32, 33, 37, 34],  // digit 3
                vec![43, 42, 38, 39, 40, 44, 41],  // digit 4
            ],
            temp_cpu_led: 51,
            temp_gpu_led: 59,
            temp_100s_digit: vec![50, 52, 47, 46, 45, 49, 48], // a=50,b=52,c=47,d=46,e=45,f=49,g=48
            temp_10s_digit: vec![58, 60, 55, 54, 53, 57, 56],  // a=58,b=60,c=55,d=54,e=53,f=57,g=56
            temp_1s_digit: vec![66, 67, 63, 62, 61, 65, 64],    // a=66,b=67,c=63,d=62,e=61,f=65,g=64
            temp_fahrenheit: 68,
            temp_celsius: 69,
            watts_w_led: 91,
            watts_digits: vec![
                vec![75, 76, 72, 71, 70, 74, 73],   // a=75,b=76,c=72,d=71,e=70,f=74,g=73
                vec![82, 83, 79, 78, 77, 81, 80],   // a=82,b=83,c=79,d=78,e=77,f=81,g=80
                vec![89, 90, 86, 85, 84, 88, 87],   // a=89,b=90,c=86,d=85,e=84,f=88,g=87
            ],
        }
    }
}

const DIGIT_SEGMENTS: [&[usize]; 10] = [
    &[0, 1, 2, 3, 4, 5, 6],
    &[1, 2],
    &[0, 1, 3, 4, 6],
    &[0, 1, 2, 3, 6],
    &[1, 2, 5, 6],
    &[0, 2, 3, 5, 6],
    &[0, 2, 3, 4, 5, 6],
    &[0, 1, 2],
    &[0, 1, 2, 3, 4, 5, 6],
    &[0, 1, 2, 3, 5, 6],
];

struct Metrics {
    cpu_temp: i32,
    cpu_usage: i32,
    cpu_speed: i32,
    cpu_watts: i32,
    gpu_temp: i32,
    gpu_usage: i32,
    gpu_speed: i32,
    gpu_watts: i32,
    last_update: Instant,
    update_interval: Duration,
    cpu_prev_idle: u64,
    cpu_prev_total: u64,
    rapl_last_energy: u64,
    rapl_last_time: Instant,
}

impl Metrics {
    fn new(update_interval_secs: f64) -> Self {
        let now = Instant::now();
        Metrics {
            cpu_temp: 0,
            cpu_usage: 0,
            cpu_speed: 0,
            cpu_watts: 0,
            gpu_temp: 0,
            gpu_usage: 0,
            gpu_speed: 0,
            gpu_watts: 0,
            last_update: now - Duration::from_secs_f64(update_interval_secs), // Force first update
            update_interval: Duration::from_secs_f64(update_interval_secs),
            cpu_prev_idle: 0,
            cpu_prev_total: 0,
            rapl_last_energy: 0,
            rapl_last_time: now,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }
        self.cpu_temp = get_cpu_temp();
        self.cpu_usage = get_cpu_usage(&mut self.cpu_prev_idle, &mut self.cpu_prev_total);
        self.cpu_speed = get_cpu_speed();
        self.cpu_watts = get_cpu_power(&mut self.rapl_last_energy, &mut self.rapl_last_time, self.cpu_usage);
        
        // Read GPU metrics
        let (gpu_temp, gpu_usage, gpu_speed, gpu_watts) = get_gpu_metrics();
        self.gpu_temp = gpu_temp;
        self.gpu_usage = gpu_usage;
        self.gpu_speed = gpu_speed;
        self.gpu_watts = gpu_watts;
        
        self.last_update = now;
    }
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn get_cpu_temp() -> i32 {
    // Prefer hwmon k10temp over thermal_zone as it's more accurate for AMD CPUs
    for i in 0..10 {
        if let Some(name) = read_file(&format!("/sys/class/hwmon/hwmon{}/name", i)) {
            let name = name.trim();
            if name == "k10temp" || name == "zenpower" || name == "coretemp" {
                // Try temp1_input first (usually CPU die)
                if let Some(temp) = read_file(&format!("/sys/class/hwmon/hwmon{}/temp1_input", i)) {
                    if let Ok(t) = temp.trim().parse::<i32>() {
                        return t / 1000;
                    }
                }
            }
        }
    }
    // Fallback to thermal_zone
    if let Some(temp) = read_file("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(t) = temp.trim().parse::<i32>() {
            return t / 1000;
        }
    }
    0
}

fn get_cpu_usage(prev_idle: &mut u64, prev_total: &mut u64) -> i32 {
    let stat = match read_file("/proc/stat") {
        Some(s) => s,
        None => return 0,
    };
    let cpu_line = match stat.lines().next() {
        Some(l) => l,
        None => return 0,
    };
    let parts: Vec<u64> = cpu_line[5..]
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() < 4 {
        return 0;
    }
    let idle = parts[3];
    let total: u64 = parts.iter().sum();
    let idle_diff = idle.saturating_sub(*prev_idle);
    let total_diff = total.saturating_sub(*prev_total);
    *prev_idle = idle;
    *prev_total = total;
    if total_diff == 0 {
        return 0;
    }
    ((total_diff - idle_diff) * 100 / total_diff) as i32
}

fn get_cpu_speed() -> i32 {
    if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        for line in contents.lines() {
            if line.starts_with("cpu MHz") {
                if let Some(mhz) = line.split(':').nth(1) {
                    if let Ok(speed) = mhz.trim().parse::<f64>() {
                        return speed as i32;
                    }
                }
            }
        }
    }
    if let Some(scaling) = read_file("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
        if let Ok(freq) = scaling.trim().parse::<i64>() {
            return (freq / 1000) as i32;
        }
    }
    0
}

fn get_gpu_metrics() -> (i32, i32, i32, i32) {
    // Returns (temp, usage, speed, watts) from nvidia-smi
    let output = match std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu,utilization.gpu,clocks.sm,power.draw", "--format=csv,noheader,nounits"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return (0, 0, 0, 0),
    };
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split(',').map(|s| s.trim()).collect();
    
    if parts.len() >= 4 {
        let temp = parts[0].parse::<i32>().unwrap_or(0);
        let usage = parts[1].parse::<i32>().unwrap_or(0);
        let speed = parts[2].parse::<i32>().unwrap_or(0);
        // Remove " W" from power if present
        let watts = parts[3].replace(" W", "").parse::<f32>().unwrap_or(0.0) as i32;
        return (temp, usage, speed, watts);
    }
    (0, 0, 0, 0)
}

fn get_cpu_power(last_energy: &mut u64, last_time: &mut Instant, cpu_usage: i32) -> i32 {
    // Try RAPL first (Intel/AMD)
    let now = Instant::now();
    if let Some(energy_str) = read_file("/sys/class/powercap/intel-rapl:0/energy_uj") {
        if let Ok(energy) = energy_str.trim().parse::<u64>() {
            let time_diff = now.duration_since(*last_time).as_secs_f64();
            if *last_energy > 0 && time_diff > 0.0 {
                let energy_diff = energy.saturating_sub(*last_energy);
                let watts = (energy_diff as f64 / 1_000_000.0) / time_diff;
                *last_energy = energy;
                *last_time = now;
                return watts.max(0.0) as i32;
            }
            *last_energy = energy;
            *last_time = now;
        }
    }
    
    // Try zenpower (AMD)
    for i in 0..10 {
        if let Some(name) = read_file(&format!("/sys/class/hwmon/hwmon{}/name", i)) {
            if name.trim() == "zenpower" {
                let mut total_watts = 0.0;
                if let Some(p1) = read_file(&format!("/sys/class/hwmon/hwmon{}/power1_input", i)) {
                    if let Ok(p) = p1.trim().parse::<i64>() {
                        total_watts += p as f64 / 1_000_000.0;
                    }
                }
                if let Some(p2) = read_file(&format!("/sys/class/hwmon/hwmon{}/power2_input", i)) {
                    if let Ok(p) = p2.trim().parse::<i64>() {
                        total_watts += p as f64 / 1_000_000.0;
                    }
                }
                return total_watts as i32;
            }
        }
    }
    
    // Fallback: estimate based on CPU usage (Idle 15W + usage% * 105W TDP)
    15 + (cpu_usage as i32 * 105 / 100)
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn interpolate_color(start: &str, end: &str, factor: f64) -> String {
    let (r1, g1, b1) = hex_to_rgb(start);
    let (r2, g2, b2) = hex_to_rgb(end);
    let r = (r1 as f64 * (1.0 - factor) + r2 as f64 * factor) as u8;
    let g = (g1 as f64 * (1.0 - factor) + g2 as f64 * factor) as u8;
    let b = (b1 as f64 * (1.0 - factor) + b2 as f64 * factor) as u8;
    format!("{:02x}{:02x}{:02x}", r, g, b)
}

fn get_color_for_value(color_spec: &str, value: i32) -> String {
    // Format: "metric;color:threshold;color:threshold" e.g., "cpu_temp;ffffff:43;ff00ff:55;0000ff:64"
    let parts: Vec<&str> = color_spec.split(';').collect();
    if parts.len() < 2 {
        return "ffffff".to_string(); // Default to white
    }
    let _metric = parts[0];
    let mut stops: Vec<(i32, String)> = Vec::new();
    for part in &parts[1..] {
        if let Some(colon_pos) = part.find(':') {
            let color = &part[..colon_pos];
            if let Ok(threshold) = part[colon_pos + 1..].parse::<i32>() {
                stops.push((threshold, color.to_string()));
            }
        }
    }
    if stops.is_empty() {
        return "ffffff".to_string();
    }
    stops.sort_by_key(|k| k.0);
    if value <= stops[0].0 {
        return stops[0].1.clone();
    }
    if value >= stops.last().unwrap().0 {
        return stops.last().unwrap().1.clone();
    }
    for i in 0..stops.len() - 1 {
        if value >= stops[i].0 && value < stops[i + 1].0 {
            let start_val = stops[i].0 as f64;
            let end_val = stops[i + 1].0 as f64;
            let factor = (value as f64 - start_val) / (end_val - start_val);
            return interpolate_color(&stops[i].1, &stops[i + 1].1, factor);
        }
    }
    "ffffff".to_string()
}

fn load_config(path: &str) -> Config {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

fn draw_digit_on_leds(leds: &mut [u8], digit: usize, segments: &[usize]) {
    if digit > 9 {
        return;
    }
    if let Some(seg_indices) = DIGIT_SEGMENTS.get(digit) {
        for &seg in *seg_indices {
            if seg < segments.len() {
                leds[segments[seg]] = 1;
            }
        }
    }
}

struct Controller {
    dev: Option<HidDevice>,
    leds: Vec<u8>,
    colors: Vec<String>,
    layout: Layout,
    config: Config,
    metrics: Metrics,
    showing_cpu: std::sync::Arc<AtomicBool>,
    metrics_updates: usize,
}

impl Controller {
    fn new() -> Self {
        let config_path = std::env::var("DIGITAL_LCD_CONFIG")
            .unwrap_or_else(|_| "config.json".to_string());
        let config = load_config(&config_path);
        let metrics = Metrics::new(config.metrics_update_interval);
        Controller {
            dev: None,
            leds: vec![0; NUMBER_OF_LEDS],
            colors: vec!["000000".to_string(); NUMBER_OF_LEDS],
            layout: Layout::default(),
            config,
            metrics,
            showing_cpu: std::sync::Arc::new(AtomicBool::new(true)),
            metrics_updates: 0,
        }
    }

    fn connect(&mut self) {
        let vendor_id = u16::from_str_radix(self.config.vendor_id.trim_start_matches("0x"), 16).unwrap_or(VENDOR_ID);
        let product_id = u16::from_str_radix(self.config.product_id.trim_start_matches("0x"), 16).unwrap_or(PRODUCT_ID);
        
        if let Ok(api) = hidapi::HidApi::new() {
            for device_info in api.device_list() {
                if device_info.vendor_id() == vendor_id && device_info.product_id() == product_id {
                    match device_info.open_device(&api) {
                        Ok(dev) => { 
                            self.dev = Some(dev);
                            return;
                        }
                        Err(e) => eprintln!("Failed to open: {:?}", e),
                    }
                }
            }
        }
        self.dev = None;
    }

    fn send_packets(&self) {
        let Some(ref dev) = self.dev else { return };
        
        let mut all_led_bytes = Vec::with_capacity(NUMBER_OF_LEDS * 3);
        for i in 0..NUMBER_OF_LEDS {
            if self.leds[i] == 0 {
                all_led_bytes.extend_from_slice(&[0, 0, 0]);
            } else {
                let color = &self.colors[i];
                let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0);
                // BRG order
                all_led_bytes.extend_from_slice(&[b, r, g]);
            }
        }

        // Packet 0: Init
        let mut p0_payload = vec![0xda, 0xdb, 0xdc, 0xdd, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0];
        while p0_payload.len() < 64 {
            p0_payload.push(0);
        }
        let p0 = std::iter::once(0u8).chain(p0_payload.iter().copied()).collect::<Vec<_>>();
        let _ = dev.write(&p0);
        thread::sleep(Duration::from_millis(10));
        
        // Packet 1+: Header + LED data
        let header: Vec<u8> = vec![
            0xda, 0xdb, 0xdc, 0xdd,
            0, 0, 0, 0, 0, 0, 0, 0,
            0x02, 0, 0, 0,
            0x59, 0x01, 0, 0,
        ];
        let full_stream: Vec<u8> = header.into_iter().chain(all_led_bytes.into_iter()).collect();
        
        for chunk in full_stream.chunks(64) {
            let mut packet = vec![0u8];
            packet.extend_from_slice(chunk);
            while packet.len() < 65 {
                packet.push(0);
            }
            let _ = dev.write(&packet);
        }
    }

    fn clear_leds(&mut self) {
        self.leds.fill(0);
    }

    fn draw_usage(&mut self, usage: i32) {
        if usage < 0 || usage > 199 {
            return;
        }
        self.leds[self.layout.usage_percent_led] = 1;
        let usage_2digit = usage % 100;
        draw_digit_on_leds(&mut self.leds, (usage_2digit % 10) as usize, &self.layout.usage_1s_digit);
        if usage >= 100 {
            draw_digit_on_leds(&mut self.leds, 0, &self.layout.usage_10s_digit);
        } else if usage_2digit >= 10 {
            draw_digit_on_leds(&mut self.leds, (usage_2digit / 10) as usize, &self.layout.usage_10s_digit);
        }
        if usage >= 100 {
            self.leds[self.layout.usage_100s_led] = 1;
        }
    }

    fn draw_speed(&mut self, speed: i32) {
        if speed < 0 || speed > 9999 {
            return;
        }
        self.leds[self.layout.speed_mhz_led] = 1;
        draw_digit_on_leds(&mut self.leds, (speed % 10) as usize, &self.layout.speed_digits[0]);
        if speed >= 10 {
            draw_digit_on_leds(&mut self.leds, ((speed / 10) % 10) as usize, &self.layout.speed_digits[1]);
        }
        if speed >= 100 {
            draw_digit_on_leds(&mut self.leds, ((speed / 100) % 10) as usize, &self.layout.speed_digits[2]);
        }
        if speed >= 1000 {
            draw_digit_on_leds(&mut self.leds, (speed / 1000) as usize, &self.layout.speed_digits[3]);
        }
    }

    fn draw_watts(&mut self, watts: i32) {
        if watts < 0 || watts > 999 {
            return;
        }
        self.leds[self.layout.watts_w_led] = 1;
        let hundreds = watts / 100;
        let tens = (watts / 10) % 10;
        let units = watts % 10;
        if watts >= 100 && !self.layout.watts_digits.is_empty() {
            draw_digit_on_leds(&mut self.leds, hundreds as usize, &self.layout.watts_digits[0]);
        }
        if watts >= 10 && self.layout.watts_digits.len() > 1 {
            draw_digit_on_leds(&mut self.leds, tens as usize, &self.layout.watts_digits[1]);
        }
        if self.layout.watts_digits.len() > 2 {
            draw_digit_on_leds(&mut self.leds, units as usize, &self.layout.watts_digits[2]);
        }
    }

    fn draw_temp(&mut self, temp: i32, is_cpu: bool, unit: &str) {
        if temp < 0 || temp > 999 {
            return;
        }
        if is_cpu {
            self.leds[self.layout.temp_cpu_led] = 1;
        } else {
            self.leds[self.layout.temp_gpu_led] = 1;
        }
        draw_digit_on_leds(&mut self.leds, (temp % 10) as usize, &self.layout.temp_1s_digit);
        if temp >= 10 {
            draw_digit_on_leds(&mut self.leds, ((temp / 10) % 10) as usize, &self.layout.temp_10s_digit);
        }
        if temp >= 100 {
            draw_digit_on_leds(&mut self.leds, (temp / 100) as usize, &self.layout.temp_100s_digit);
        }
        if unit == "celsius" {
            self.leds[self.layout.temp_celsius] = 1;
        } else {
            self.leds[self.layout.temp_fahrenheit] = 1;
        }
    }

    fn update_colors(&mut self) {
        let usage_color_spec = self.config.usage.colors.first().cloned().unwrap_or_default();
        let color = get_color_for_value(&usage_color_spec, self.metrics.cpu_temp);
        for i in 0..NUMBER_OF_LEDS {
            self.colors[i] = color.clone();
        }
    }

    fn display_cpu_watts(&mut self) {
        self.clear_leds();
        self.update_colors();
        let cpu_temp = if self.config.cpu_temperature_unit == "fahrenheit" {
            self.metrics.cpu_temp * 9 / 5 + 32
        } else {
            self.metrics.cpu_temp
        };
        self.draw_usage(self.metrics.cpu_usage);
        self.draw_speed(self.metrics.cpu_speed);
        self.draw_watts(self.metrics.cpu_watts);
        let unit = self.config.cpu_temperature_unit.clone();
        self.draw_temp(cpu_temp, true, &unit);
    }

    fn display_loop(&mut self) {
        let update_interval = Duration::from_secs_f64(self.config.update_interval);
        let cycle_duration = (self.config.cycle_duration / self.config.update_interval) as usize;
        loop {
            self.metrics.update();
            self.config = load_config(
                &std::env::var("DIGITAL_LCD_CONFIG").unwrap_or_else(|_| "../config.json".to_string())
            );
            if self.dev.is_none() {
                self.connect();
                if self.dev.is_none() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
            match self.config.display_mode.as_str() {
                "cpu_watts" => {
                    self.display_cpu_watts();
                }
                "alternating_watts" => {
                    self.metrics_updates += 1;
                    if self.metrics_updates >= cycle_duration {
                        self.metrics_updates = 0;
                        let current = self.showing_cpu.load(Ordering::SeqCst);
                        self.showing_cpu.store(!current, Ordering::SeqCst);
                    }
                    self.clear_leds();
                    self.update_colors();
                    let (usage, speed, watts, temp, is_cpu) = if self.showing_cpu.load(Ordering::SeqCst) {
                        (self.metrics.cpu_usage, self.metrics.cpu_speed, self.metrics.cpu_watts, self.metrics.cpu_temp, true)
                    } else {
                        (self.metrics.gpu_usage, self.metrics.gpu_speed, self.metrics.gpu_watts, self.metrics.gpu_temp, false)
                    };
                    let display_temp = if (is_cpu && self.config.cpu_temperature_unit == "fahrenheit") ||
                        (!is_cpu && self.config.gpu_temperature_unit == "fahrenheit") {
                        temp * 9 / 5 + 32
                    } else {
                        temp
                    };
                    let unit = if is_cpu { 
                        self.config.cpu_temperature_unit.clone() 
                    } else { 
                        self.config.gpu_temperature_unit.clone() 
                    };
                    self.draw_usage(usage);
                    self.draw_speed(speed);
                    self.draw_watts(watts);
                    self.draw_temp(display_temp, is_cpu, &unit);
                }
                "gpu_watts" => {
                    self.clear_leds();
                    self.update_colors();
                    let gpu_temp = if self.config.gpu_temperature_unit == "fahrenheit" {
                        self.metrics.gpu_temp * 9 / 5 + 32
                    } else {
                        self.metrics.gpu_temp
                    };
                    self.draw_usage(self.metrics.gpu_usage);
                    self.draw_speed(self.metrics.gpu_speed);
                    self.draw_watts(self.metrics.gpu_watts);
                    let unit = self.config.gpu_temperature_unit.clone();
                    self.draw_temp(gpu_temp, false, &unit);
                }
                "debug_ui" => {
                    self.clear_leds();
                    for i in 0..NUMBER_OF_LEDS {
                        self.leds[i] = 1;
                        self.colors[i] = "00ff00".to_string();
                    }
                }
                _ => {
                    self.display_cpu_watts();
                }
            }
            self.send_packets();
            thread::sleep(update_interval);
        }
    }
}

fn main() {
    println!("Thermal LCD Controller - Rust Edition");
    println!("Connecting to device...");
    let mut controller = Controller::new();
    controller.connect();
    if controller.dev.is_none() {
        eprintln!("Failed to connect to HID device. Make sure the device is connected.");
    } else {
        println!("Device connected! Starting display loop...");
    }
    controller.display_loop();
}
