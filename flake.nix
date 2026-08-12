{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      inherit (pkgs) lib;

      nodejs = pkgs.nodejs_24;
      pnpm = pkgs.pnpm.override {inherit nodejs;};
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "clippy"
          "rustfmt"
          "rust-src"
          "rust-analyzer"
        ];
      };

      source = ./.;

      rustPackage = name:
        pkgs.rustPlatform.buildRustPackage {
          pname = "yin-${name}";
          version = "0.1.0";
          src = source;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = ["--bin" name];
        };

      rustCheck = {
        name,
        command,
      }:
        pkgs.rustPlatform.buildRustPackage {
          pname = "yin-${name}";
          version = "0.1.0";
          src = source;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [rust];
          buildPhase = command;
          installPhase = "touch $out";
          doCheck = false;
        };

      api = rustPackage "api";
      bot = rustPackage "bot";
      migrate = rustPackage "migrate";

      authPnpmDeps = pkgs.fetchPnpmDeps {
        pname = "yin-auth-pnpm-deps";
        version = "0.1.0";
        src = source;
        fetcherVersion = 2;
        hash = "sha256-p5BaGAqzTdNYOVVBeLfsliupqRKnQkHEbyV8UfNfsKE=";
      };

      auth = pkgs.stdenvNoCC.mkDerivation {
        pname = "yin-auth";
        version = "0.1.0";
        src = source;
        pnpmDeps = authPnpmDeps;

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
        inherit api bot migrate auth apiImage botImage migrateImage authImage;
        default = botImage;
      };

      checks = {
        inherit api bot migrate auth apiImage botImage migrateImage authImage;

        rust-fmt = pkgs.runCommand "yin-rust-fmt" {nativeBuildInputs = [rust];} ''
          cd ${source}
          cargo fmt --all --check
          touch $out
        '';

        rust-clippy = rustCheck {
          name = "rust-clippy";
          command = ''
            runHook preBuild
            cargo clippy --workspace --all-targets --locked --offline -- -D warnings
            runHook postBuild
          '';
        };

        rust-tests = rustCheck {
          name = "rust-tests";
          command = ''
            runHook preBuild
            cargo test --workspace --locked --offline
            runHook postBuild
          '';
        };
      };

      devShells.default = with pkgs;
        mkShell {
          packages = [
            nodejs
            pnpm
            openssl
            just
            mprocs
            rust
            kind
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          '';
        };
    });
}
