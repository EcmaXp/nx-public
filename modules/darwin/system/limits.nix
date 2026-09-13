{ config, lib, ... }:
lib.nx.gate config.nx.darwin.system.limits {
  launchd.daemons.limit-maxfiles = {
    serviceConfig = {
      Label = "limit.maxfiles";
      ProgramArguments = [
        "/bin/launchctl"
        "limit"
        "maxfiles"
        "4096"
        "32768"
      ];
      RunAtLoad = true;
    };
  };
}
