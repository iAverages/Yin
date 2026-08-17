# Dashboard layout

`apps/panel/src/routes/_dashboard.tsx` is the authenticated, pathless layout for
all dashboard routes: `/`, `/guilds/$guildId` (general settings, social embeds and
custom commands), and `/guilds/$guildId/moderation` (punishment ladder). The trailing
underscore in `_dashboard.guilds.$guildId_.moderation.tsx` makes moderation a sibling
of the settings page rather than nesting one editor inside the other. Switching
guilds from moderation stays on moderation; both pages share the settings cache. Add new dashboard routes beneath `_dashboard`; do not add
another sidebar or duplicate the auth guard. The hidden `/admin` area also uses
this shell, with an additional owner-only parent guard and independent API checks;
see [admin.md](admin.md) for its authorization, telemetry and deployment contract.
Login remains outside this layout.
API authorization is independent of the route guard.

The layout owns the inset content area, breadcrumb, desktop sidebar toggle and
mobile bottom sheet. `DashboardSidebar` contains the guild selector, navigation, bottom
"Add a server" link and account dropdown with sign-out. Kobalte supplies menu
keyboard navigation and dismissal. Corvu Drawer supplies the Solid-native bottom
sheet with drag-to-dismiss, an animated backdrop, focus trapping and restoration.
The sheet respects reduced motion and safe-area insets, and closes on desktop
resize. Mobile dropdowns portal inside the sheet so they share its focus boundary;
Escape dismisses an open menu before the sheet.
There are no placeholder account/billing actions.

The guild selector is a searchable popover containing only manageable servers where
Yin is installed. Search is local, case-insensitive and reset on opening. The input
keeps normal text-editing behavior; arrow keys focus guild links and Enter follows
them. Non-installed servers remain available on the main server list. Both popovers
animate on entry/exit. The desktop sidebar slides and collapses its layout space;
its hidden contents are inert and any open popup closes with it. The sidebar stays
fixed to the viewport while the page scrolls. Ctrl+B toggles the desktop sidebar
(or the mobile sheet on narrow screens). Collapsing removes the content frame's
outer margins, rounded corners and ring, giving it the full viewport width.
All motion respects
`prefers-reduced-motion`. Pending actions disable controls without changing labels.

The sidebar and server list share user-scoped TanStack Query options. Settings
cache keys include both user and guild IDs. Editors remount on guild changes and
capture the guild ID for in-flight saves, so a completed save cannot update another
server's cache. Save and delete results use `solid-sonner` toasts rather than inline
notices. Successes appear only after the API succeeds; failures are dismissible error
toasts. Each toast names its server so late saves remain clear after navigation.
The shared layout owns the toaster.
Sign-out clears the query cache.

## SSR and loading

The shared auth guard runs before rendering. The sidebar, account details and page
frame render on the server. Guild queries are enabled during SSR, with local Solid
Suspense boundaries for the selector and page data. SSR-only route loaders prefetch
the guild directory and page settings in parallel, sharing a **2-second render
deadline** created only after authentication succeeds. `SSR_DATA_BUDGET_MS` and
`waitForSsrData` live in `apps/panel/src/lib/ssr-data.ts`.

- Data ready within the budget renders as actual cards/forms/tables in the first HTML.
- At the deadline, ready sections remain visible and pending sections render skeletons.
- The original requests keep running in the request-scoped QueryClient. Solid streams
  their results into the existing boundaries, without cancelling or duplicating fetches.
- Later loaders get only the remaining budget, not another 2 seconds. Timers are
  cleared on completion, and late promise failures remain handled.

The render deadline is not an API error and does not change the existing 10-second
request timeout. Real failures use local error/retry UI, including failures arriving
after the skeleton has been sent. Authentication is never subject to this fallback.
TanStack may still wait for the complete render for bot user agents.

Client-side loaders do not proxy or block on these reads; components continue
fetching the API directly using TanStack Query. Only the toaster remains client-only.
Skeletons also cover uncached client navigation and respect reduced-motion preferences.

`api-client.ts` uses an explicit isomorphic boundary to forward the incoming cookie
to the configured API **only during SSR**. Redirects are rejected, and requests are
bounded by a timeout. Browser reads, refreshes and writes still fetch the API
directly with credentials - there is no guild server-function proxy. Authenticated
HTML remains private/no-store. QueryClient is created per router/request, never
globally. `@tanstack/solid-router-ssr-query` dehydrates the request's cache into
the HTML and streams queries still pending when the SSR data deadline passes.

Read query data inside its local boundary rather than gating resource access on
`isSuccess`/`isError`: those flags can still reflect the initial optimistic state
during SSR. Loading and failures must not suspend or replace the surrounding
navigation. In the browser, reads retry server, timeout and network errors up to
three times with backoff; client errors (auth, not found, validation) fail
immediately. SSR never retries. Failed reads then show an explicit retry action.
Route pending components cover client-side route loading; the same skeletons
handle uncached API requests. Background refresh keeps existing
content visible. Cached settings/moderation navigation needs no additional request.

Controls that need handlers stay disabled until hydration; form state is initialized
on the server too. This avoids visible-but-inert clicks and empty SSR input values.
Content ready within the SSR budget is visible with JavaScript entirely disabled.
Past the deadline, Solid's inline streaming scripts replace skeletons as data arrives;
the application bundle is not needed to render that content. Hydration adds
interactivity and reuses the server-populated query cache without refetching.

## Production deployment

The Nix flake builds `panel` (the self-contained Nitro Node server and public
assets) and `panelImage` (a non-root Node.js 26 image listening on port 3000).
The build workflow publishes `ghcr.io/iaverages/yin/panel:production-<commit>`
alongside the API, auth, bot and migration images.

```sh
nix build .#panelImage
```

Production hosts are:

- Panel: `https://yin.kirsi.dev`
- API: `https://api-yin.kirsi.dev`
- Auth: `https://auth-yin.kirsi.dev`

`VITE_API_URL` and `VITE_AUTH_URL` are set in the panel derivation in `flake.nix`.
Vite embeds them in both the browser and SSR bundles at **build time**; changing
container environment variables cannot override them. Rebuild the image when
changing origins. SSR must be able to reach the public API and auth HTTPS hosts.
`HOST` and `PORT` remain runtime settings (defaults: `0.0.0.0:3000`).

Kubernetes manifests are authored in `~/takina-deployment/config/apps/yin.nix`,
with hostnames in `config/clusters/takina.nix`. The panel shares the existing
migration-gated Flux stage and uses `/login` for unauthenticated startup/readiness
checks. Update the deployment repository's `config/images.nix` revision only
after CI publishes all five images, then regenerate its YAML.

Auth must use `BETTER_AUTH_URL=https://auth-yin.kirsi.dev`,
`AUTH_COOKIE_DOMAIN=kirsi.dev`, and trust the three production origins. The API
must allow credentialed CORS from `https://yin.kirsi.dev`. The shared cookie
domain means all `kirsi.dev` subdomains must be trusted. Set Discord's OAuth
callback to `https://auth-yin.kirsi.dev/api/auth/callback/discord` and provision
DNS/HTTPS routing for all three hosts before reconciliation.

### Discord install results

The panel serves public `/install/discord/success` and `/install/discord/error`
pages outside the dashboard auth guard. Both render without JavaScript or a
session. The error page can restart the guild or user install through the auth
service; returning to the dashboard still requires sign-in. These are result
screens, not proof of a session or of the bot's guild membership.

Auth requires explicit callback URLs (there is no auth-host fallback):

- `AUTH_INSTALL_SUCCESS_URL=https://yin.kirsi.dev/install/discord/success`
- `AUTH_INSTALL_ERROR_URL=https://yin.kirsi.dev/install/discord/error`

For local Kubernetes, both URLs use `http://panel.yin.localhost`. The callback
origin must be included in `AUTH_TRUSTED_ORIGINS`. Deploy the panel routes before
switching an existing auth service to these URLs. Install entry points and the
Discord OAuth callback remain on the auth host.

## Code style

Panel functions use arrow syntax. Run `pnpm --filter panel lint` for Oxlint and
`pnpm --filter panel format` to apply Oxfmt's four-space indentation. Both lint
and formatting checks run in `nix flake check`. The generated `routeTree.gen.ts`
remains generator-owned and is excluded from these checks.

## Browser tests

CI runs the unit and browser suites through Nix checks; see [testing.md](testing.md)
for the full test matrix, standalone check commands and failure artifacts.

From the repository root:

```sh
pnpm --dir apps/panel exec playwright install chromium
pnpm --dir apps/panel test:e2e
pnpm --dir apps/panel test:unit
```

Tests run headlessly and start an isolated panel on `127.0.0.1:3010`, fixture auth
on `127.0.0.1:33010`, and a read-only SSR API fixture on `127.0.0.1:33011`.
Browser API calls are intercepted by Playwright. No real Discord credentials or
development database writes are used. All three ports must be free; fixture servers
are only started by tests, never imported by production code. The headless browser
uses a normal Chrome user agent so full-content SSR is tested without relying on
special bot buffering.

On systems using a separately installed Chromium (such as NixOS), set
`PLAYWRIGHT_CHROMIUM_EXECUTABLE` to its executable path.

Tests cover shared layout persistence, card order and labels, guild switching,
account menu keyboard/outside-click behavior, sign-out and retry, mobile drawer
focus trapping and touch dismissal, authenticated route boundaries, pending/failed
saves and toasts, installed-only guild search, popover animations, fixed scrolling,
Ctrl+B, the edge-to-edge collapsed layout, full-content SSR with JavaScript disabled
(including responses within the deadline), request-scoped hydration, client-navigation
skeletons, retries, and moderation navigation/cache sharing. Deadline tests cover
slow lists/settings/moderation, independently ready sections, late failures and
single, uncancelled API requests. Unit tests use mocked timers to verify the exact
budget, shared deadlines, timer cleanup, and early/late rejection handling.
Traces and screenshots are saved under `apps/panel/test-results/` on failure.
