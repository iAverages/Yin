# Tests

## Nix / CI

On Linux, run the same checks as CI with:

```sh
nix flake check --print-build-logs --keep-going
```

`.github/workflows/build.yaml` runs this on pull requests and pushes to `main`.
Test checks are defined in `nix/checks.nix`:

| Check | Runs |
| --- | --- |
| `rust-tests` | Workspace Rust tests, including tests marked `#[ignore]`, and doc tests |
| `auth-tests` | Auth service TypeScript tests |
| `panel-unit` | Panel SSR-deadline unit tests |
| `panel-lint` | Oxlint over panel source, tests and configuration; warnings fail the check |
| `panel-format` | Oxfmt check with four-space indentation |
| `panel-e2e` | All configured Playwright tests in headless Chromium (Linux only) |

The flake also checks Rust formatting, Clippy, and the production packages/images.
Image publishing only happens after the checks succeed. Nix may reuse a successful
check from its cache when its inputs have not changed.

The Rust check starts a private MariaDB inside the build sandbox and runs
`cargo test --workspace -- --include-ignored`. SQLx creates and migrates an
isolated database for each database-backed test. The check supplies its own
`DATABASE_URL` and stops MariaDB on success or failure. No external database,
Docker daemon, or production credentials are needed.

Playwright uses Nix-provided Chromium and fonts, with browser downloads disabled.
Its existing web-server fixtures start the panel and fake API/auth services inside
the sandbox; tests do not contact Discord. The JS checks use the lockfile-backed
pnpm dependency derivation and do not install packages from the network at test
time. Nix can still download dependencies and cached build outputs beforehand.

Run individual checks with:

```sh
nix build .#checks.x86_64-linux.rust-tests --no-link -L
nix build .#checks.x86_64-linux.auth-tests --no-link -L
nix build .#checks.x86_64-linux.panel-unit --no-link -L
nix build .#checks.x86_64-linux.panel-e2e -L --keep-failed
```

Use `aarch64-linux` instead of `x86_64-linux` on ARM Linux. Successful browser
checks retain their HTML report and test results under the output directory
(`result/playwright-report` and `result/test-results` with the last command).
For failures, `--keep-failed` preserves the build directory printed by Nix;
reports, screenshots and traces are under its `*/apps/panel/` directory. CI
uploads these as the `panel-test-results` artifact when available.

## Without Nix checks

```sh
cargo test --workspace
pnpm --filter @yin/auth test
pnpm --filter panel test:unit
pnpm --filter panel test:e2e
pnpm --filter panel lint
pnpm --filter panel format:check
```

Run `pnpm --filter panel format` to apply formatting. Panel-local Oxlint and
Oxfmt configuration lives in `apps/panel/.oxlintrc.json` and `.oxfmtrc.json`.
Generated route trees, lockfiles and build/test output are excluded from formatting;
the generated route tree is also excluded from linting. Handwritten panel functions
use arrows; route components are initialized before the route definitions that
reference them.

Plain `cargo test` still skips ignored tests. To include database-backed tests,
provide a disposable MariaDB server and credentials with permission to create
test databases:

```sh
DATABASE_URL='mysql://test:test@127.0.0.1:3306/mysql' \
  cargo test --workspace -- --include-ignored
```

For Playwright outside the Nix check, install its Chromium browser or set
`PLAYWRIGHT_CHROMIUM_EXECUTABLE` to an existing Chromium executable. See
[panel.md](panel.md) for fixture ports and panel-specific test details.
