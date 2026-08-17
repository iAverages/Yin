# Panel guild directory

The guild list has two sources of truth:

- `bot_guilds`: persistent bot membership and current guild metadata. `feature-guilds`
  handles READY, GUILD_CREATE, GUILD_UPDATE and GUILD_DELETE. Temporary unavailability
  is not removal. READY reconciles each shard so departures while offline are removed.
  Guild settings are retained when the bot leaves.
- `user_guild_cache`: a **five-minute**, user-scoped OAuth membership and permission
  snapshot. After storing the OAuth account and creating a session, Better Auth
  calls the API's existing authenticated `GET /guilds` to warm it. The API alone
  fetches and paginates Discord's guild endpoint with `limit=200`, owns the JSON
  format and TTL, and writes the cache. An unexpired snapshot is reused, including
  on sign-in; missing or expired snapshots refresh on the next guild read.
  Failed lookups never replace a snapshot with a partial result or extend its expiry.

We cache all of the user's guild memberships, including non-installed guilds. Even
when a guild is in `bot_guilds`, we still need evidence that this particular user
belongs to it and has Manage Server, Administrator, or owner permissions. Bot
membership alone must never grant access. Installed guild names/icons come from
`bot_guilds`; non-installed metadata comes from the OAuth snapshot.

## Read and write behavior

- Guild list and settings reads reuse the unexpired snapshot, avoiding Discord calls
  during normal navigation. Read permission changes may take up to five minutes to
  appear. An expired snapshot is not used when refresh fails.
- All settings mutations fetch fresh Discord membership/permissions **for the
  selected guild only**, using `limit=1&after=<guild_id - 1>`. The returned ID must
  exactly match: if the user left, Discord may return the next guild instead.
  A cached read cannot authorize a write. Mutations also require the bot to be
  installed. Targeted lookups never overwrite the complete cached snapshot.
  This avoids calculating permissions for every guild on each save. A development
  comparison with 180 memberships took 1,804 ms for the full list versus 174 ms
  for the targeted lookup. Discord rate limits and network latency still apply.
- The bot-installed flag is joined from the database on every API read; it is not
  frozen into the user's five-minute snapshot. A join/leave changes classification
  without requiring another OAuth lookup.
- The panel uses a 30-second TanStack Query stale time. "Refresh servers" refetches
  the API to pick up installations immediately; it does not bypass the OAuth TTL.
- Installed servers sort first, alphabetically, and link to settings. Non-installed
  servers appear darker, with an "Add Yin" label and selected-guild install link.
- External installation links go to the auth service and then Discord. In-app
  navigation uses TanStack Link. The panel's shared layout and browser tests are
  described in [panel.md](panel.md).

## Deployment

Run `cargo run -p migrate` before deploying the updated auth/API/bot services.
Migration `0008_guild_directory.sql` adds both tables. Restart/reconnect the bot to
populate its initial directory. No re-login is required for existing sessions,
although their first guild request may perform one Discord lookup.

The auth service uses `API_SERVICE_URL` (default `http://api:3000`) for this
server-to-server request. Both Kubernetes deployments set it to the internal API
service. For auth running outside Kubernetes, set it to the reachable API origin.
Use HTTPS if this traffic crosses an untrusted network.

The hook signs the newly persisted session's cookie using Better Auth's cookie
name and signing key. The API validates it through the normal auth middleware;
there is no user-ID override, shared service token, or unauthenticated warm-up
endpoint. Session validation does not create another session or recurse into the
hook. Redirects are rejected to avoid forwarding the cookie to another host.

Warm-up is awaited with an eight-second request deadline. Failures only produce a
generic warning and do not fail sign-in; the next panel request retries a missing
or expired cache. Auth no longer reads Discord tokens or writes guild snapshots.
All guild-fetching/cache behavior lives in `crates/feature-guilds`.

## Tests

- `cargo test -p feature-guilds --lib`
- `pnpm --dir apps/auth test`
- Database integration tests (use a disposable MariaDB instance and a user with
  CREATE DATABASE privileges; SQLx creates isolated test databases):
  `DATABASE_URL=... cargo test -p feature-guilds --test directory -- --ignored`
