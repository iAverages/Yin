{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      inherit (pkgs) lib;

      nodejs = pkgs.nodejs_26;
      pnpm = pkgs.pnpm;

      source = ./.;
      craneLib = crane.mkLib pkgs;
      rustSource = lib.cleanSourceWith {
        src = source;
        filter = path: type:
          craneLib.filterCargoSources path type || lib.hasSuffix ".sql" path;
      };
      commonRustArgs = {
        pname = "yin";
        version = "0.1.0";
        src = rustSource;
        strictDeps = true;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonRustArgs;
      workspaceArtifacts = craneLib.cargoBuild (commonRustArgs
        // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--workspace --bins";
        });

      rustPackage = name:
        craneLib.buildPackage (commonRustArgs
          // {
            pname = "yin-${name}";
            cargoArtifacts = workspaceArtifacts;
            cargoExtraArgs = "--bin ${name}";
            doCheck = false;
          });

      api = rustPackage "api";
      bot = rustPackage "bot";
      migrate = rustPackage "migrate";

      pnpmDeps = pkgs.fetchPnpmDeps {
        pname = "yin-pnpm-deps";
        version = "0.1.0";
        src = source;
        inherit pnpm;
        fetcherVersion = 4;
        hash = "sha256-L5e9sGFuR+cd6K2L8eiryEdPxOpmWb7V/PqBl2ndJLQ=";
      };

      auth = pkgs.stdenvNoCC.mkDerivation {
        pname = "yin-auth";
        version = "0.1.0";
        src = source;
        inherit pnpmDeps;

        nativeBuildInputs = [
          nodejs
          pnpm
          pkgs.pnpmConfigHook
        ];

        buildPhase = ''
          runHook preBuild
          pnpm auth:build
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -r apps package.json pnpm-lock.yaml pnpm-workspace.yaml node_modules "$out/"
          runHook postInstall
        '';
      };

      panel = pkgs.stdenvNoCC.mkDerivation {
        pname = "yin-panel";
        version = "0.1.0";
        src = source;
        inherit pnpmDeps;

        nativeBuildInputs = [
          nodejs
          pnpm
          pkgs.pnpmConfigHook
        ];

        VITE_API_URL = "https://api-yin.kirsi.dev";
        VITE_AUTH_URL = "https://auth-yin.kirsi.dev";
        NITRO_PRESET = "node-server";

        buildPhase = ''
          runHook preBuild
          pnpm --filter panel build
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -r apps/panel/.output/. "$out/"
          runHook postInstall
        '';
      };

      panelImage = pkgs.dockerTools.buildLayeredImage {
        name = "yin-panel";
        tag = "latest";
        contents = [nodejs pkgs.cacert];
        extraCommands = ''
          mkdir -m 1777 tmp
        '';
        config = {
          Cmd = ["${nodejs}/bin/node" "${panel}/server/index.mjs"];
          Env = [
            "HOST=0.0.0.0"
            "PORT=3000"
            "NODE_ENV=production"
            "NODE_EXTRA_CA_CERTS=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
          ExposedPorts."3000/tcp" = {};
          User = "10001:10001";
          WorkingDir = panel;
        };
      };

      serviceImage = {
        name,
        package,
        port ? null,
        env ? [],
      }:
        pkgs.dockerTools.buildLayeredImage {
          name = "yin-${name}";
          tag = "latest";
          contents = [package pkgs.cacert];
          extraCommands = ''
            mkdir -m 1777 tmp
          '';
          config = {
            Cmd = ["${package}/bin/${name}"];
            Env =
              [
                "RUST_LOG=info"
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ]
              ++ env;
            ExposedPorts = lib.optionalAttrs (port != null) {"${toString port}/tcp" = {};};
            User = "10001:10001";
            WorkingDir = "/";
          };
        };

      apiImage = serviceImage {
        name = "api";
        package = api;
        port = 3000;
        env = ["API_BIND_ADDR=0.0.0.0:3000"];
      };

      botImage = serviceImage {
        name = "bot";
        package = bot;
        env = ["APP_ENV=production"];
      };

      migrateImage = serviceImage {
        name = "migrate";
        package = migrate;
      };

      authImage = pkgs.dockerTools.buildLayeredImage {
        name = "yin-auth";
        tag = "latest";
        contents = [pkgs.deno pkgs.cacert];
        extraCommands = ''
          mkdir -m 1777 tmp
        '';
        config = {
          Cmd = [
            "${pkgs.deno}/bin/deno"
            "run"
            "--allow-env"
            "--allow-net"
            "${auth}/apps/auth/src/index.ts"
          ];
          Env = [
            "AUTH_HOST=0.0.0.0"
            "AUTH_PORT=3001"
            "NODE_ENV=production"
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
          ExposedPorts."3001/tcp" = {};
          User = "10001:10001";
          WorkingDir = auth;
        };
      };
    in {
      packages = {
        inherit api bot migrate auth panel apiImage botImage migrateImage authImage panelImage;
        default = botImage;
      };

      checks =
        {
          inherit api bot migrate auth panel apiImage botImage migrateImage authImage panelImage;

          rust-fmt = craneLib.cargoFmt {
            pname = "yin-rust-fmt";
            version = "0.1.0";
            src = rustSource;
          };

          rust-clippy = craneLib.cargoClippy (commonRustArgs
            // {
              pname = "yin-rust-clippy";
              cargoArtifacts = workspaceArtifacts;
              cargoClippyExtraArgs = "--workspace --all-targets -- --deny warnings";
            });
        }
        // import ./nix/checks.nix {
          inherit pkgs lib craneLib commonRustArgs source nodejs pnpm pnpmDeps;
          cargoArtifacts = workspaceArtifacts;
        };

      devShells.default = craneLib.devShell {
        packages = with pkgs; [
          nodejs
          pnpm
          openssl
          just
          mprocs
          rust-analyzer
          kind
          tailwindcss-language-server
        ];

        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
        '';
      };
    });
}
