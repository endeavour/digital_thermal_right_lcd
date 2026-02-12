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
  ];

  # Simple postInstall - just copy config and use runCommand pattern
  postInstall = ''
    mkdir -p $out/bin
    mkdir -p $out/share/hid-digital-lcd-controller
    cp ${./config.json} $out/share/hid-digital-lcd-controller/config.json
    
    # Create wrapper using runCommand to avoid quoting issues
    writeShellScriptBin "hid-digital-lcd-controller" ''
      #!${stdenv.shell}
      exec ${python3}/bin/python3 -m digital_thermal_right_lcd.controller --config "$out/share/hid-digital-lcd-controller/config.json"
    ''
    in
    writeShellScriptBin "hid-digital-lcd-controller" ''
      #!${stdenv.shell}
      exec ${python3}/bin/python3 -c "${pythonCode}"
    ''
    
    chmod +x $out/bin/hid-digital-lcd-controller
  '';

  meta = with lib; {
    description = "Digital LCD controller for Thermalright CPU coolers";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}