use hidapi::HidApi;
fn main() {
    let api = HidApi::new().unwrap();
    for dev in api.device_list() {
        println!("{:04x}:{:04x} - {:?}", dev.vendor_id(), dev.product_id(), dev.path());
    }
}
