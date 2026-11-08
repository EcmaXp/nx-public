{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.aws {
  home.sessionVariables = {
    AWS_DEFAULT_OUTPUT = "json";
    AWS_DEFAULT_REGION = "ap-northeast-2";
    AWS_EC2_METADATA_DISABLED = "true";
    AWS_REGION = "ap-northeast-2";
  };
}
