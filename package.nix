{ lib
, python3
, stdenv
, hidapi
, libdrm
, libpciaccess
, linuxHeaders
, mesa
, libGL
, gcc
, pkg-config
, zlib
, glibc
}:

python3.pkgs.buildPythonPackage {
  pname = "hid-digital-lcd-controller";
  version = "1.0";
  format = "pyproject";

  src = ./.;

  nativeBuildInputs = [
    pkg-config
    python3.pkgs.setuptools
    python3.pkgs.wheel
  ];

  buildInputs = [
    hidapi
    libdrm
    libpciaccess
    linuxHeaders
    mesa
    libGL
    gcc
    zlib
    glibc
  ];

  propagatedBuildInputs = with python3.pkgs; [
    numpy
    hid
    psutil
    pyamdgpuinfo
  ];

  preBuild = ''
    export CPPFLAGS="-I${libdrm.dev}/include -I${linuxHeaders}/include/drm -I${libdrm.dev}/include/libdrm"
    export C_INCLUDE_PATH="${libdrm.dev}/include:${linuxHeaders}/include:$C_INCLUDE_PATH"
    export CPLUS_INCLUDE_PATH="${libdrm.dev}/include:${linuxHeaders}/include:$CPLUS_INCLUDE_PATH"
    export PKG_CONFIG_PATH="${libdrm.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib:${glibc}/lib:${zlib}/lib:${hidapi}/lib:$LD_LIBRARY_PATH"
  '';

  pythonImportsCheck = [
    "numpy"
    "hid"
    "psutil"
    "pyamdgpuinfo"
  ];

  # Install wrapper script
  postInstall = ''
    mkdir -p $out/bin
    mkdir -p $out/share/hid-digital-lcd-controller
    
    # Copy default config
    cp ${./config.json} $out/share/hid-digital-lcd-controller/config.json
    
    # Create wrapper script
    cat > $out/bin/hid-digital-lcd-controller << 'EOF'
#!/usr/bin/env python3
import sys
import os

# Add the package to Python path
sys.path.insert(0, '${python3.sitePackages}')

# Import and run the controller
from controller import main

if __name__ == '__main__':
    config_path = sys.argv[1] if len(sys.argv) > 1 else '$out/share/hid-digital-lcd-controller/config.json'
    main(config_path)
EOF
    
    chmod +x $out/bin/hid-digital-lcd-controller
  '';

  meta = with lib; {
    description = "Digital LCD controller for Thermalright CPU coolers";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}