################################################################################
# Declares a launchd service for nix-darwin.
################################################################################
{ config, lib, pkgs, ... }: let
  cfg = config.services.sytter;
  default-user = "sytter";
  default-group = "sytter";
  service-name = "sytter";
in {
  options.services.sytter = {

    enable = lib.mkEnableOption "Sytter, IFTTT for a host.";

    user = lib.mkOption {
      type = lib.types.str;
      default = default-user;
      example = "yourUser";
      description = ''
        The user to run Sytter as.
        By default, a user named `${default-user}` will be created.
      '';
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = default-group;
      example = "yourGroup";
      description = ''
        The group to run Sytter as.
        By default, a group named `${default-group}` will be created.
      '';
    };

    data-path = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/${service-name}";
      description = ''
        Location where Sytters are stored.
      '';
    };

    log-file = lib.mkOption {
      type = lib.types.str;
      default = "${cfg.data-path}/logs";
      description = ''
        Location where logs for the service are stored.
      '';
    };

    log-level = lib.mkOption {
      default = "info";
      description = ''
        The verbosity level to use for logging.
      '';
      example = lib.literalExpression ''"debug"'';
      type = lib.types.enum [
        "error"
        "warn"
        "info"
        "debug"
      ];
    };

    sytters = lib.mkOption {
      default = {};
      description = ''
        Sytters to ensure for the Sytter service to use.
      '';
      defaultText = lib.literalExpression ''
        {
          name: "Bluetooth disabled on Sleep";
          description: ''$
            Disable Bluetooth on sleep, and enable it again when waking up \
            Helpful for macOS, which is notorious for draining battery from
            chatty Bluetooth devices.
          ''$;
          triggers = [
            { kind = "power"; events = ["Sleep", "Wake"]; }
          ];
          conditions = [
            { kind = "shell"; script = "true"; }
          ];
          executors = [
            {
              kind = "shell";
              script = ''$
                if [[ "$sytter_power_event" == \"Sleep\" ]]; then
                  sytter_bluetooth_enabled_at_sleep=$(blueutil --power)
                  sytter-var-write sytter_bluetooth_enabled_at_sleep
                  blueutil --power 0
                else
                  sytter-vars sytter_bluetooth_enabled_at_sleep
                  blueutil --power $sytter_bluetooth_enabled_at_sleep
                fi
              ''$;
            }
          ];
        }
      '';
      type = lib.types.attrs;
    };
  };
  config = lib.mkIf cfg.enable {
    launchd.user.agents.sytter = {
      command = "${pkgs.sytter}/bin/sytter";
      serviceConfig = {
        KeepAlive = true;
        RunAtLoad = true;
        ProcessType = "Standard";
        StandardOutPath = cfg.log-file;
        StandardErrorPath = cfg.log-file;
        EnvironmentVariables = {
          SYTTERS_PATH = "${cfg.data-path}/sytters";
          SYTTER_VERBOSITY = cfg.log-level;
        };
      };
    };
  };
}
