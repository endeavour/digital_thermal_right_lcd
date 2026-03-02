{ lib
, rustPlatform
, pkg-config
, systemd
, udev
}:

rustPlatform.buildRustPackage {
  pname = "thermal-lcd";
  version = "0.1.0";

  src = ./thermal_lcd;
  cargoLock = {
    lockFile = ./thermal_lcd/Cargo.lock;
  };

  buildInputs = [ systemd.dev udev ];
  nativeBuildInputs = [ pkg-config ];

  postInstall = ''
    mkdir -p $out/share/thermal-lcd
    cp ${./config.json} $out/share/thermal-lcd/config.json
    mv $out/bin/thermal_lcd $out/bin/thermal-lcd || true
  '';

  meta = with lib; {
    description = "Thermal LCD Controller for Phantom Spirit 120 EVO";
    platforms = platforms.linux;
  };
}
