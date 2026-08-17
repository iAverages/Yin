{
  pkgs,
  lib,
  craneLib,
  commonRustArgs,
  cargoArtifacts,
  source,
  nodejs,
  pnpm,
  pnpmDeps,
}: let
  jsTest = name: command:
    pkgs.stdenvNoCC.mkDerivation {
      pname = "yin-${name}";
      version = "0.1.0";
      src = source;
      inherit pnpmDeps;
      nativeBuildInputs = [nodejs pnpm pkgs.pnpmConfigHook];
      CI = "true";
      PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
      preConfigure = ''
        export HOME="$TMPDIR/home"
        mkdir -p "$HOME"
      '';
      buildPhase = ''
        runHook preBuild
        ${command}
        runHook postBuild
      '';
      installPhase = ''
        mkdir -p "$out"
      '';
    };
in
  {
    rust-tests = craneLib.cargoTest (commonRustArgs
      // {
        pname = "yin-rust-tests";
        inherit cargoArtifacts;
        cargoTestExtraArgs = "--workspace -- --include-ignored";
        nativeBuildInputs = [pkgs.mariadb];
        preCheck = ''
          mysql_dir="$TMPDIR/mysql"
          mysql_socket="$TMPDIR/mysql.sock"
          mysql_log="$TMPDIR/mysql.log"
          if ! mariadb-install-db --no-defaults --datadir="$mysql_dir" \
            --auth-root-authentication-method=normal --skip-test-db > "$mysql_log" 2>&1; then
            tail -n 200 "$mysql_log"
            exit 1
          fi
          mariadbd --no-defaults --datadir="$mysql_dir" \
            --socket="$mysql_socket" --pid-file="$TMPDIR/mysql.pid" \
            --bind-address=127.0.0.1 --port=33306 --skip-name-resolve \
            --innodb-buffer-pool-size=64M --log-error="$mysql_log" &
          mysql_pid=$!

          stopDatabase() {
            kill "$mysql_pid" 2>/dev/null || true
            wait "$mysql_pid" 2>/dev/null || true
          }
          trap 'status=$?; if [ "$status" -ne 0 ]; then tail -n 200 "$mysql_log"; fi; stopDatabase; exit "$status"' EXIT

          ready=false
          for attempt in $(seq 1 300); do
            if mariadb --no-defaults --socket="$mysql_socket" --user=root \
              --execute='SELECT 1' > /dev/null 2>&1; then
              ready=true
              break
            fi
            if ! kill -0 "$mysql_pid" 2>/dev/null; then
              break
            fi
            sleep 0.1
          done
          if [ "$ready" != true ]; then
            echo "Test MariaDB failed to start" >&2
            exit 1
          fi

          mariadb --no-defaults --socket="$mysql_socket" --user=root --execute="
            CREATE USER 'yin_test'@'127.0.0.1' IDENTIFIED BY 'test';
            GRANT ALL PRIVILEGES ON *.* TO 'yin_test'@'127.0.0.1';
          "
          export DATABASE_URL='mysql://yin_test:test@127.0.0.1:33306/mysql'
        '';
        postCheck = ''
          stopDatabase
          trap - EXIT
        '';
      });

    auth-tests = jsTest "auth-tests" "pnpm --filter @yin/auth test";
    panel-unit = jsTest "panel-unit" "pnpm --filter panel test:unit";
    panel-lint = jsTest "panel-lint" "pnpm --filter panel lint";
    panel-format = jsTest "panel-format" "pnpm --filter panel format:check";
  }
  // lib.optionalAttrs pkgs.stdenv.isLinux {
    panel-e2e =
      (jsTest "panel-e2e" ''
        pnpm --filter panel exec playwright test --reporter=list,html
      '').overrideAttrs {
        PLAYWRIGHT_CHROMIUM_EXECUTABLE = lib.getExe pkgs.chromium;
        PLAYWRIGHT_HTML_OPEN = "never";
        FONTCONFIG_FILE = pkgs.makeFontsConf {
          fontDirectories = [pkgs.dejavu_fonts pkgs.liberation_ttf];
        };
        installPhase = ''
          mkdir -p "$out"
          cp -r apps/panel/playwright-report apps/panel/test-results "$out/"
        '';
      };
  }
