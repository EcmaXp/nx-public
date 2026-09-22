{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins {
  programs.broot.enable = true;
  programs.carapace.enable = true;
  programs.eza.enable = true;
  programs.eza.enableBashIntegration = false;
  programs.eza.enableZshIntegration = false;
  programs.eza.git = true;
  programs.fzf.enable = true;
  programs.navi.enable = true;
  programs.skim.enable = true;
  programs.tmux.enable = true;
  programs.watson.enable = true;
  programs.yazi.enable = true;
  programs.yazi.shellWrapperName = "yy";
  programs.zellij.enable = true;
  programs.zoxide.enable = true;
}
