{ pkgs, ... }:

{
  packages = [ pkgs.git ];

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  pre-commit.hooks.clippy.enable = true;
}
