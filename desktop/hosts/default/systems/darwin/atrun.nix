{
  launchd.daemons.atrun = {
    serviceConfig = {
      ProgramArguments = [ "/usr/libexec/atrun" ];
      StartInterval = 30;
    };
  };
}
