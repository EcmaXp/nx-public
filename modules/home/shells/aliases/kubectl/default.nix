{
  config,
  lib,
  pkgs,
  ...
}:
let
  kubectlAbbr = pkgs.runCommand "kubectl-abbr" { } ''
    grep '^abbr --add ' ${./kubectl_aliases.fish} > abbrs.txt

    mkdir -p $out/functions
    names=$(awk '{ print $3 }' abbrs.txt | sed 's/\./\\./g' | paste -sd'|')
    printf "abbr --add __nx_kubectl --regex '^(%s)\$' --function __nx_kubectl_abbr\n" "$names" > $out/init.fish

    {
      echo 'function __nx_kubectl_abbr'
      echo '    switch $argv[1]'
      awk '{ name = $3; sub(/^abbr --add [^ ]+ /, ""); printf "        case %s\n            printf %%s %s\n", name, $0 }' abbrs.txt
      echo "        case '*'"
      echo '            return 1'
      echo '    end'
      echo 'end'
    } > $out/functions/__nx_kubectl_abbr.fish

    test "$(grep -c '^        case ' $out/functions/__nx_kubectl_abbr.fish)" = "$(( $(wc -l < abbrs.txt) + 1 ))"
  '';
in
lib.nx.gate config.nx.home.shells.aliases.kubectl {
  xdg.configFile = {
    "fish/functions/__nx_kubectl_abbr.fish" = {
      source = "${kubectlAbbr}/functions/__nx_kubectl_abbr.fish";
    };
  };

  programs.fish = {
    interactiveShellInit = ''
      source ${kubectlAbbr}/init.fish
    '';
  };
}
