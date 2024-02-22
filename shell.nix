{ pkgs ? import <nixpkgs> { } }:
(pkgs.buildFHSUserEnv {
  name = "pipzone";
  targetPkgs = pkgs: (with pkgs; [
    python39
    python39Packages.pip
    python39Packages.virtualenv
    maturin
  ]);
  runScript = "bash";
  profile = ''
    virtualenv .venv
    source .venv/bin/activate
  '';
}).env
