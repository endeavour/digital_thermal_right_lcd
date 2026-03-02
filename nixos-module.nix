{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.thermal-lcd;
  thermal-lcd-pkg = pkgs.callPackage ./package.nix {};
in {
  options.services.thermal-lcd = {
    enable = mkEnableOption "Thermal LCD Controller for Phantom Spirit 120 EVO";

    config = mkOption {
      type = types.path;
      default = "${thermal-lcd-pkg}/share/thermal-lcd/config.json";
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
    systemd.packages = [ thermal-lcd-pkg ];

    systemd.services.thermal-lcd = {
      description = "Thermal LCD Controller for Phantom Spirit 120 EVO";
      after = [ "network.target" ];
      wants = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${thermal-lcd-pkg}/bin/thermal-lcd";
        Restart = "always";
        RestartSec = 5;
        StandardOutput = "journal";
        StandardError = "journal";
      };

      environment = {
        DIGITAL_LCD_CONFIG = cfg.config;
        PATH = lib.mkForce "/run/current-system/sw/bin:/nix/var/nix/profiles/default/bin:/usr/bin:/bin";
      };
    };

    services.udev.extraRules = ''
      KERNEL=="hidraw*", ATTRS{idVendor}=="0416", ATTRS{idProduct}=="8001", MODE="0660", GROUP="${cfg.group}"
      SUBSYSTEM=="powercap", MODE="0666"
    '';
  };
}
