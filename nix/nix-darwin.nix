################################################################################
# Declares a launchd service for nix-darwin.
################################################################################
{ config, lib, pkgs, ... }: let
  cfg = config.services.sytter;
  default-user = "sytter";
  default-group = "sytter";
  service-name = "sytter";
  toml = pkgs.formats.toml { };
  sytter-type = lib.types.submodule {
    options = {
      name = lib.mkOption {
        type = lib.types.str;
        description = "Human-readable name for this sytter.";
      };
      description = lib.mkOption {
        type = lib.types.str;
        default = "";
        description = "Human-readable description of what this sytter does.";
      };
      trigger = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        description = "One or more triggers that fire this sytter.";
      };
      condition = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        default = [ ];
        description = "Conditions that gate execution when a trigger fires.";
      };
      execute = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        description = "Actions to perform when triggered and conditions pass.";
      };
      failure = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        default = [ ];
        description = "Handlers to invoke when an executor fails.";
      };
    };
  };
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
        Location where Sytter runtime data (logs, state) is stored.
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
      default = { };
      description = ''
        Sytters to manage for the Sytter service.  Each attribute name is used
        as the TOML filename stem.  The sytter binary is pointed directly at
        the resulting Nix store path, so no activation script is needed.
      '';
      example = lib.literalExpression ''
        {
          bluetooth-on-sleep = {
            name = "Bluetooth disabled on sleep";
            description = "Disable Bluetooth on sleep, enable it again on wake.";
            trigger = [
              { kind = "power"; events = [ "Sleep" "Wake" ]; }
            ];
            condition = [
              { kind = "shell"; script = "true"; }
            ];
            execute = [
              {
                kind = "shell";
                script = '''
                  if [[ "$sytter_power_event" == "Sleep" ]]; then
                    sytter_bluetooth_enabled_at_sleep=$(blueutil --power)
                    sytter-var-write sytter_bluetooth_enabled_at_sleep
                    blueutil --power 0
                  else
                    sytter-vars sytter_bluetooth_enabled_at_sleep
                    blueutil --power "$sytter_bluetooth_enabled_at_sleep"
                  fi
                ''';
              }
            ];
          };
        }
      '';
      type = lib.types.attrsOf sytter-type;
    };
  };
  config = lib.mkIf cfg.enable {
    launchd.user.agents.sytter = let
      sytters-pkg = pkgs.linkFarm "sytter-configs" (
        lib.mapAttrsToList (name: sytter: {
          name = "${name}.toml";
          path = toml.generate "${name}.toml" sytter;
        }) cfg.sytters
      );
    in {
      command = "${pkgs.sytter}/bin/sytter";
      serviceConfig = {
        KeepAlive = true;
        RunAtLoad = true;
        ProcessType = "Standard";
        StandardOutPath = cfg.log-file;
        StandardErrorPath = cfg.log-file;
        EnvironmentVariables = {
          SYTTERS_PATH = "${sytters-pkg}";
          SYTTER_VERBOSITY = cfg.log-level;
        };
      };
    };
  };
}
