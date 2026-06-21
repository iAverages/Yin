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
```

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
```

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
pnpm dlx @better-auth/cli@latest generate \
  --cwd apps/auth \
  --config src/auth.ts \
  --output ../../crates/database/migrations/0002_better_auth.sql \
  --yes
```

Then rerun:

```bash
DATABASE_URL=mysql://yin:yin@localhost:3306/yin cargo run -p migrate
```

## Live Update Behavior

Rust services run through `cargo watch`. Tilt syncs source changes into the pod, and `cargo watch` restarts the relevant Rust process. Cargo incremental build cache is stored in an `emptyDir` mounted at `/workspace/target` while the pod exists.

Auth service syncs TypeScript source changes into the pod. `tsx watch` reloads the auth server without a full image rebuild.

Traefik dashboard is port-forwarded at:

```text
http://localhost:8080/dashboard/
```
