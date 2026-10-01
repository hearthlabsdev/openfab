{
  description = "LaunchSpace Kiosk";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        python = pkgs.python313;
        pythonEnv = python.withPackages (ps: with ps; [
          flask
          jinja2
          gunicorn
          pyprusalink
        ]);
      in {
        packages.default = pkgs.writeShellApplication {
            name = "launchspace-kiosk";

          runtimeInputs = [
            pythonEnv
            pkgs.sqlite
          ];

          text = ''
            export FLASK_APP=server.py
            exec flask run "$@"
          '';
        };
        devShells.default = pkgs.mkShell {
          buildInputs = [
            (python.withPackages (ps: with ps; [
              flask
              jinja2
              gunicorn
            ]))
            pkgs.sqlite
          ];

          shellHook = ''
            export FLASK_APP=server.py
            export FLASK_ENV=development

            if [ ! -d .venv ]; then
              echo "Development shell ready."
            fi
          '';
        };
      });
}