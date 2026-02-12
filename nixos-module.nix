{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.hid-digital-lcd-controller;
in {
  options.services.hid-digital-lcd-controller = {
    enable = mkEnableOption "Digital LCD Controller for Thermalright CPU Cooler";

    config = mkOption {
      type = types.path;
      default = ./config.json;
      description = "Path to the configuration file";
    };

    user = mkOption {
      type = types.str;
      default = "root";
      description = "User to run the service as";
    };

    group = mkOption {
      type = types.str;
      default = "root";
      description = "Group to run the service as";
    };
  };

  config = mkIf cfg.enable {
    systemd.packages = [ config._module.args.package ];

    systemd.services.hid-digital-lcd-controller = {
      description = "Digital LCD Controller for Thermalright CPU Cooler";
      after = [ "network.target" ];
      wants = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${config._module.args.package}/bin/hid-digital-lcd-controller ${cfg.config}";
        Restart = "always";
        RestartSec = 5;
        StandardOutput = "journal";
        StandardError = "journal";
      };

      environment = {
        CPPFLAGS = "-I${pkgs.libdrm.dev}/include -I${pkgs.linuxHeaders}/include/drm -I${pkgs.libdrm.dev}/include/libdrm";
        C_INCLUDE_PATH = "${pkgs.libdrm.dev}/include:${pkgs.linuxHeaders}/include:$C_INCLUDE_PATH";
        CPLUS_INCLUDE_PATH = "${pkgs.libdrm.dev}/include:${pkgs.linuxHeaders}/include:$CPLUS_INCLUDE_PATH";
        PKG_CONFIG_PATH = "${pkgs.libdrm.dev}/lib/pkgconfig:$PKG_CONFIG_PATH";
        LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.glibc}/lib:${pkgs.zlib}/lib:${pkgs.hidapi}/lib:$LD_LIBRARY_PATH";
      };
    };

    # Add udev rule for device access
    services.udev.extraRules = ''
      KERNEL=="hidraw*", ATTRS{idVendor}=="0416", ATTRS{idProduct}=="8001", MODE="0660", GROUP="${cfg.group}"
    '';
  };
}