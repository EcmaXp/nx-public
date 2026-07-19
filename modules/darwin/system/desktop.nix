{ config, lib, ... }:
lib.nx.gate config.nx.darwin.system.desktop {
  time.timeZone = "Asia/Seoul";
}
