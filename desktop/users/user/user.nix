{
  imports = [
    ./scripts
    ./secrets
  ];

  programs.git = {
    userName = "user";
    userEmail = "user@example.com";
  };
}
