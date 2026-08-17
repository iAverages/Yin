# Owner diagnostics

Open **/admin** on the panel after signing in with Discord account
**307952129958477824**. There is deliberately no entry in the sidebar, guild
selector, or account menu. The page uses the existing dashboard shell and is
read-only. Admin pages live beneath the guarded `_dashboard.admin` layout; any
future admin page must stay under `/admin`.

## Authorization

The API is the authorization boundary. All endpoints in the `/admin` router:

1. Validate the existing Better Auth session through the auth service.
2. Query the session user's linked `account` row, requiring
   `providerId = 'discord'` and `accountId = '307952129958477824'`.
3. Return private, no-store responses, including errors.

The internal Better Auth user ID is **not** a Discord ID. Neither matching that
internal ID to the allowlist nor possessing guild owner/Administrator permissions
grants access. A missing session returns 401; authenticated non-owners get 404.
Authorization/service/database failures fail closed. The linked account is checked
on every request; revocation does not wait for the session or guild cache to expire.

The panel's parent guard calls `GET /admin/access` before loading any diagnostic
data, during both SSR and client navigation. Non-owners see a not-found page.
The API independently authorizes direct requests, so hiding navigation is not a
security mechanism. Diagnostic query keys include the session user ID, and the
existing sign-out flow clears the query cache. No bot tokens, OAuth tokens, session
tokens, environment variables, or raw gateway payloads are included.

## API (on the API origin)

- `GET /admin/access` - returns `true` for the authorized owner.
- `GET /admin/guilds` - every current membership in `bot_guilds`, with name,
  Discord ID, icon hash and shard ID. IDs serialize as strings. This is independent
  of the owner's OAuth guild list and settings configuration.
- `GET /admin/state` - process snapshots from the last 24 hours, newest first,
  with per-instance liveness and snapshot age.

Guild membership is maintained by the existing gateway event directory.
Unavailable guilds are not departures; names can be pending after READY. The
directory is retained while the bot is offline and reconciled on reconnect, so it
is not proof of current gateway health. See [guild-directory.md](guild-directory.md).

## Runtime telemetry

The bot and API are separate processes. The bot writes a diagnostic snapshot into
`bot_runtime` every 15 seconds using their shared database, without a new network
listener or internal credential. Each process boot gets a new UUID, so restarts
and overlapping processes remain distinguishable.

Snapshots report:
- Bot identity (unknown before READY), environment and package version.
- Process uptime **at the snapshot**, not a continuously extrapolated uptime.
- Discord's total shard count (unknown before READY), locally running shards,
  gateway connection stages and heartbeat acknowledgement latency.
- Cached guild/user/channel counts and unavailable guild count. Cached users are
  **not** total members, and local runner count is not necessarily total shards.

The publisher starts independently of Poise setup, including while connecting.
The shard-manager lock is only held while copying runner metadata, never during
database IO. Writes are bounded to five seconds; failures log a warning without
stopping the gateway. Graceful exit marks the snapshot stopped.

A process is `reporting` only when its latest snapshot is less than 45 seconds old.
This does not mean its shards are connected. At 45 seconds it becomes `stale`,
which may indicate a crash, database outage, or stalled publisher - not definitive
proof that Discord disconnected. Stale/stopped gateway statuses are explicitly
labeled historical. The browser continues aging snapshots during request failures
and while no fresh response arrives. Empty snapshots are shown as unknown/missing,
never a healthy zero-shard bot.

Runtime reads refresh every 15 seconds while the page is active; guild reads refresh
every minute. Both have manual refresh/retry controls. Slow or failed reads stay
inside local boundaries using the panel's shared two-second SSR data budget.
Guild search is local and case-insensitive, by name or exact/partial Discord ID.

Records older than 24 hours are excluded from reads and pruned by the publisher.
This is a short-lived diagnostic view, not a metrics history or logging system.

## Deployment

Run migrations **before** deploying the new bot/API:

```sh
cargo run -p migrate
```

Migration `0009_bot_runtime.sql` adds the runtime snapshot table. Deploy/restart
the bot, API and panel together. No new configuration variables are needed.
Snapshots should appear within 15 seconds of a running bot connecting to the
database. If the table is missing, diagnostics fail visibly; existing gateway
operation is not made dependent on a successful telemetry write.

## Tests

```sh
cargo check -p bot -p api
cargo test -p api
# Disposable MariaDB; the test user needs CREATE DATABASE permission:
DATABASE_URL=... cargo test -p api -- --ignored
pnpm --dir apps/panel exec tsc --noEmit
pnpm --dir apps/panel test:e2e tests/admin.spec.ts
```

The database-backed tests exercise the actual API router, linked-account checks
on every endpoint, missing sessions, ID/provider spoofing, account revocation,
database failures, full guild membership, string snowflakes, heartbeat persistence,
stopped/stale states and retention. Browser fixtures never connect to Discord or
write development data. Browser tests cover SSR without JavaScript, hidden
navigation, denial, search, unknown metadata, refresh failures/retry, stale aging,
empty states, hydration and mobile layout. See [panel.md](panel.md) for browser setup.
