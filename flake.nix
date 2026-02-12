{
  description = "Digital LCD Controller for Thermalright CPU Coolers";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: 
    let
      system = "x86_64-linux";  # adjust if you're on a different arch
      pkgs = nixpkgs.legacyPackages.${system};
      
      # Package definition
      hid-digital-lcd-controller = pkgs.callPackage ./package.nix {};
    in {
      # Package for use in other Nix configurations
      packages.${system}.default = hid-digital-lcd-controller;
      
      # NixOS module
      nixosModules.default = import ./nixos-module.nix;

      # Development shell (unchanged)
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          python313
          python3Packages.tkinter
          libdrm.dev
          libpciaccess
          linuxHeaders
          mesa
          libGL
          gcc
          pkg-config
          zlib
          python3Packages.cython
          uv
          stdenv.cc.cc.lib
          glibc
          hidapi
        ];
        
        shellHook = ''
          export CPPFLAGS="-I${pkgs.linuxHeaders}/include/drm -I${pkgs.libdrm.dev}/include -I${pkgs.libdrm.dev}/include/libdrm"
          export C_INCLUDE_PATH="${pkgs.libdrm.dev}/include:${pkgs.linuxHeaders}/include:$C_INCLUDE_PATH"
          export CPLUS_INCLUDE_PATH="${pkgs.libdrm.dev}/include:${pkgs.linuxHeaders}/include:$CPLUS_INCLUDE_PATH"
          export PKG_CONFIG_PATH="${pkgs.libdrm.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.glibc}/lib:${pkgs.zlib}/lib:${pkgs.hidapi}/lib:$LD_LIBRARY_PATH"
          echo "Development environment ready with DRM headers"
          echo "Try: uv sync"
          echo "For device access, use: sudo -E uv run src/controller.py config.json"
          echo "Or update your NixOS udev rule:"
          echo 'services.udev.extraRules = '''
          echo '    KERNEL=="hidraw*", ATTRS{idVendor}=="0416", ATTRS{idProduct}=="8001", MODE="0660", GROUP="wheel"'
          echo "'''"
        '';
      };

      # Example configuration for testing
      nixosConfigurations.test-vm = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          self.nixosModules.default
          ({ pkgs, ... }: {
            services.hid-digital-lcd-controller.enable = true;
            
            # Use the config from this repo
            services.hid-digital-lcd-controller.config = ./config.json;
            
            # Optional: run as a specific user
            services.hid-digital-lcd-controller.user = "root";
            services.hid-digital-lcd-controller.group = "root";
            
            system.stateVersion = "24.05";
          })
        ];
      };
    };
}

