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
      triggers = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        description = "One or more triggers that fire this sytter.";
      };
      conditions = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        default = [ ];
        description = "Conditions that gate execution when a trigger fires.";
      };
      executors = lib.mkOption {
        type = lib.types.listOf lib.types.attrs;
        description = "Actions to perform when triggered and conditions pass.";
      };
      failures = lib.mkOption {
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

    http-port = lib.mkOption {
      default = 8080;
      type = lib.types.port;
      description = ''
        Port for Sytter's internal HTTP/IPC server.  Override it when the
        default collides with another service on the host (Sytter exits if it
        cannot bind).
      '';
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
            triggers = [
              { kind = "power"; events = [ "Sleep" "Wake" ]; }
            ];
            conditions = [
              { kind = "shell"; script = "true"; }
            ];
            executors = [
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
    # launchd does not create the parent directory of StandardOutPath, and a
    # user LaunchAgent runs as the console user (not root), so create the
    # data/log directory and hand ownership to that user — otherwise the
    # agent's stdout/stderr have nowhere to go.  postActivation is used because
    # nix-darwin only runs its known activation phases, not arbitrarily-named
    # entries.
    system.activationScripts.postActivation.text = lib.mkAfter ''
      mkdir -p ${cfg.data-path}
      chown ${config.system.primaryUser} ${cfg.data-path}
    '';
    launchd.user.agents.sytter = let
      sytters-pkg = pkgs.linkFarm "sytter-configs" (
        lib.mapAttrsToList (name: sytter: {
          name = "${name}.toml";
          path = toml.generate "${name}.toml" sytter;
        }) cfg.sytters
      );
    in {
      # Pass the config path and log level as CLI flags.  The binary reads
      # only --sytters-path / --log-level; it ignores the SYTTERS_PATH and
      # SYTTER_VERBOSITY env vars, so without these it falls back to its
      # default ~/.config/sytter/sytters (which does not exist) and panics.
      command =
        "${pkgs.sytter}/bin/sytter"
        + " --sytters-path ${sytters-pkg}"
        + " --log-level ${cfg.log-level}";
      serviceConfig = {
        KeepAlive = true;
        RunAtLoad = true;
        ProcessType = "Standard";
        StandardOutPath = cfg.log-file;
        StandardErrorPath = cfg.log-file;
        EnvironmentVariables = {
          # The binary reads the HTTP port only from this (lowercase) env var.
          sytter_http_port = toString cfg.http-port;
        };
      };
    };
  };
}
