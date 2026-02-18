{ specialArgs, ... }:
{
  home-manager = {
    useUserPackages = true;
    extraSpecialArgs = specialArgs;
  };
}
