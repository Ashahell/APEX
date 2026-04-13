# L2: Nix Flake for APEX
# Usage: nix run .#apex
{
  description = "APEX - Autonomous Agent Platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.apex = pkgs.buildEnv {
          name = "apex";
          paths = [
            pkgs.rustc
            pkgs.cargo
            pkgs.nodejs_20
            pkgs.python311
            pkgs.poetry
            pkgs.docker
          ];
        };

        packages.default = self.packages.${system}.apex;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            nodejs_20
            python311
            poetry
            docker
            just
          ];

          APEX_USE_LLM = "0";
        };
      }
    );
}
