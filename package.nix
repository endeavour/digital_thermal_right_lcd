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
    python3.pkgs.hatchling
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

  # Install wrapper script that sets up proper environment
  postInstall = ''
    mkdir -p $out/bin
    mkdir -p $out/share/hid-digital-lcd-controller
    
    # Copy default config
    cp ${./config.json} $out/share/hid-digital-lcd-controller/config.json
    
    # Create wrapper script that sets up environment
    cat > $out/bin/hid-digital-lcd-controller << EOF
#!/usr/bin/env bash
set -e

# Set environment variables for native libraries
export CPPFLAGS="-I${libdrm.dev}/include -I${linuxHeaders}/include/drm -I${libdrm.dev}/include/libdrm"
export C_INCLUDE_PATH="${libdrm.dev}/include:${linuxHeaders}/include:\$C_INCLUDE_PATH"
export CPLUS_INCLUDE_PATH="${libdrm.dev}/include:${linuxHeaders}/include:\$CPLUS_INCLUDE_PATH"
export PKG_CONFIG_PATH="${libdrm.dev}/lib/pkgconfig:\$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib:${glibc}/lib:${zlib}/lib:${hidapi}/lib:\$LD_LIBRARY_PATH"

# Run the controller
config_file="''${1:-$out/share/hid-digital-lcd-controller/config.json}"
exec ${python3}/bin/python3 -c "
import sys
sys.path.insert(0, '$out/lib/python3.13/site-packages')
from controller import main
main('$config_file')
"
EOF
    
    chmod +x $out/bin/hid-digital-lcd-controller
  '';

  meta = with lib; {
    description = "Digital LCD controller for Thermalright CPU coolers";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}