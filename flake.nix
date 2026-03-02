{
  description = "Thermal LCD Controller for Phantom Spirit 120 EVO (Rust)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      overlay = final: prev: {
        thermal-lcd = prev.rustPlatform.buildRustPackage {
          pname = "thermal-lcd";
          version = "0.1.0";
          src = ./thermal_lcd;
          cargoLock = {
            lockFile = ./thermal_lcd/Cargo.lock;
          };
          buildInputs = [ prev.systemd.dev prev.udev ];
          nativeBuildInputs = [ prev.pkg-config ];
          postInstall = ''
            mkdir -p $out/share/thermal-lcd
            cp ${./config.json} $out/share/thermal-lcd/config.json
          '';
          meta = with prev.lib; {
            description = "Thermal LCD Controller for Phantom Spirit 120 EVO";
            platforms = platforms.linux;
          };
        };
      };
    in
    {
      overlays.default = overlay;

      lib = (flake-utils.lib.eachSystem systems (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default overlay ]; };
        in {
          thermal-lcd-src = pkgs.thermal-lcd.src;
        }
      )) // {
        x86_64-linux = (flake-utils.lib.eachSystem systems (system:
          let pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default overlay ]; };
          in { thermal-lcd-src = pkgs.thermal-lcd.src; }
        )).x86_64-linux;
      };

      nixosModules.default = import ./nixos-module.nix;
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          overlays = [ rust-overlay.overlays.default overlay ];
          inherit system;
        };
      in
      {
        packages.thermal-lcd = pkgs.thermal-lcd;
        packages.default = pkgs.thermal-lcd;

        devShells.${system} = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            cargo
            pkg-config
            systemd.dev
          ];
        };
      }
    );
}
