{ config, lib, ... }:
lib.nx.gate config.nx.darwin.system.atrun {
  launchd.daemons.atrun = {
    serviceConfig = {
      ProgramArguments = [ "/usr/libexec/atrun" ];
      StartInterval = 30;
    };
  };
}
