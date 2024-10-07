{ pkgs ? import <nixpkgs> { } }:
(pkgs.buildFHSUserEnv {
  name = "pipzone";
  targetPkgs = pkgs: (with pkgs; [
    python312
    python312Packages.pip
    python312Packages.virtualenv
    python312Packages.pytest
    python312Packages.pyperf
    maturin
  ]);
  runScript = "bash";
  profile = ''
    virtualenv .venv
    source .venv/bin/activate
  '';
}).env
