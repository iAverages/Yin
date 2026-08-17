# Local Kubernetes Development

This project uses kind and Tilt for local Kubernetes development.

## Prerequisites

- Docker
- kind
- kubectl
- tilt
- pnpm
- Rust toolchain

## Create The Cluster

Create the local cluster once with host ports mapped to Traefik's NodePorts:

```bash
kind create cluster --name yin --config ./k8s/yin-kind.yaml
```

The `Tiltfile` is pinned to the `kind-yin` Kubernetes context.

## Ingress

Tilt runs Traefik inside the `yin-dev` namespace. No separate ingress-nginx install is required.

Ingress hosts:

```text
http://api.yin.localhost
http://auth.yin.localhost
http://panel.yin.localhost
```

The panel's Service uses an EndpointSlice targeting the host-side IPv4 gateway of
Docker's `kind` network. Tilt discovers this address and populates the EndpointSlice
in `k8s/local/panel.yaml`. Vite listens on `0.0.0.0:3000` so Traefik can reach it.

## Configure Secrets

Create a local secret manifest:

```bash
cp k8s/local/secret.yaml.example k8s/local/secret.yaml
```

Fill in:

```text
DISCORD_TOKEN
DISCORD_DEV_GUILD_ID
BETTER_AUTH_SECRET
DISCORD_CLIENT_ID
DISCORD_CLIENT_SECRET
```

PostHog is optional. To enable feature flag evaluation, set
`POSTHOG_PROJECT_TOKEN`. To manage flags with `!admin flag`, also set
`POSTHOG_PROJECT_ID` and `POSTHOG_PERSONAL_API_KEY`.

The PostHog personal API key needs `feature_flag:read` and `feature_flag:write`
scopes so bot owners can manage flags with:

```text
!admin flag global <key> <value>
!admin flag user <key> <value> <user-id>
!admin flag guild <key> <value> <guild-id>
!admin flag member <key> <value> <user-id> <guild-id>
```

Boolean flags accept `true` or `false`; multivariate flags accept a configured variant key.
More specific settings take precedence: member, user, guild, then global.

`k8s/local/secret.yaml` is ignored by git.

## Start Tilt

```bash
tilt up
```

Tilt runs:

```text
traefik
mariadb
bot
api
auth
panel
```

The panel runs locally as a Tilt-managed Vite development server at
<http://localhost:3000> and is routed through Traefik at
<http://panel.yin.localhost>. Tilt installs workspace dependencies before starting
the panel and provides its links and readiness status in the dashboard.

The API is available at <http://api.yin.localhost> and is also port-forwarded at
<http://localhost:3003>.

Tilt does not run database migrations.

## Run Migrations Manually

MariaDB is exposed on localhost port 3306 through the kind port mapping. Tilt also port-forwards the same port while it is running.

```text
mysql://yin:yin@localhost:3306/yin
```

Run migrations manually:

```bash
DATABASE_URL=mysql://yin:yin@localhost:3306/yin cargo run -p migrate
```

## Better Auth Migrations

Generate Better Auth SQL into the Rust database migrations directory:

```bash
MIGRATION_TIMESTAMP=$(date -u +%Y%m%d%H%M%S)
pnpm dlx @better-auth/cli@latest generate \
  --cwd apps/auth \
  --config src/auth.ts \
  --output "../../crates/database/migrations/${MIGRATION_TIMESTAMP}_better_auth.sql" \
  --yes
```

Migration filenames use UTC `YYYYMMDDHHMMSS_description.sql` prefixes. Never rename or edit a
migration after it has been applied; add a new timestamped migration instead.

Then rerun:

```bash
DATABASE_URL=mysql://yin:yin@localhost:3306/yin cargo run -p migrate
```

## Live Update Behavior

Rust services run through `cargo watch`. Tilt syncs source changes into the pod, and each watcher
only observes crates used by that service. Cargo incremental build caches use separate local-node
`hostPath` volumes for the bot and API, so pod rollouts do not cause cold rebuilds. Delete
`/var/lib/yin-dev/cargo-target` inside the Kind node when a clean Rust rebuild is required.

Auth service syncs TypeScript source changes into the pod. `tsx watch` reloads the auth server without a full image rebuild.

The panel uses Vite's native HMR for source and CSS changes, including through the
Traefik hostname. Tilt watches only the workspace package manifests, lockfile, and
workspace configuration for this local resource, so source edits do not restart
its process. Dependency changes rerun `pnpm install --frozen-lockfile` and restart
Vite; update the root `pnpm-lock.yaml` with `pnpm install` when changing
dependencies. Panel files are excluded from the auth image build context.

This follows Tilt's [local server setup with `serve_cmd` and readiness probes](https://docs.tilt.dev/local_resource.html).

Traefik dashboard is port-forwarded at:

```text
http://localhost:8080/dashboard/
```
