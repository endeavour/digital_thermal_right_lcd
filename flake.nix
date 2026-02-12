{
  description = "FHS shell for uv + pyamdgpuinfo build";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: 
    let
      system = "x86_64-linux";  # adjust if you're on a different arch
      pkgs = nixpkgs.legacyPackages.${system};
    in {
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
    };
}

