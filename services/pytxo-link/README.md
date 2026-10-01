# Pytxo Link

Optional Pytxo account/control API for entitlements, organization policy, hosted
usage accounting, and provider-neutral commerce reconciliation. Local Core does
not require this service. The monorepo client is
[`HttpBillingReconciler`](../../crates/pytxo-core/src/billing/link_reconciler.rs).

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| GET | `/v1/entitlements/status` | Current tier and agent limits |
| GET | `/v1/wallet/balance` | Ultra wallet balance (microcredits) |
| POST | `/v1/runs/start` | Run started envelope |
| POST | `/v1/runs/end` | Run ended + usage totals |
| POST | `/v1/routing/workspace-grants` | Experimental hosted-recipient workspace grant under a signed account session |
| DELETE | `/v1/routing/workspace-grants` | Revoke that hosted workspace grant and invalidate its token revision |
| POST | `/v1/routing/tokens` | Experimental account-and-workspace-bound routing credential; disabled by default |
| DELETE | `/v1/routing/tokens` | Revoke that routing credential |
| POST | `/v1/routing/desktop-authorizations` | Signed browser session issues a one-use PKCE code |
| POST | `/v1/routing/desktop-exchange` | Desktop redeems code, state and verifier for a routing-only credential |
| GET / DELETE | `/v1/routing/desktop-session` | Read or disconnect that Desktop routing credential |
| DELETE | `/v1/routing/desktop-sessions` | Signed browser session revokes all Desktop routing access, grants and evaluation tokens |
| POST | `/v1/webhooks/paddle` | Signed Paddle subscription events |
| POST | `/v1/webhooks/dodo` | Signed Dodo subscription events |

## Production defaults

| Variable | Default | Purpose |
|----------|---------|---------|
| `LINK_BIND` | `127.0.0.1:8787` | Listen address (local) |
| `PORT` | Railway injects | Hosted listen port |
| `LINK_API_KEY` | unset (loopback dev only) | Bearer token for API routes |
| `LINK_REQUIRE_AUTH` | `true` when an API key or JWKS validator is configured | Reject unsigned requests; mandatory for any non-loopback or `PORT` bind |
| `DATABASE_URL` | unset | Postgres for entitlements, policies, usage, and durable commerce events |
| `LINK_ROUTING_TOKEN_EXPERIMENT` | unset | Set to `1` only for the disabled-by-default routing token experiment |
| `LINK_ROUTING_GRANT_EXPERIMENT` | unset | Set to `1` only with the token experiment to permit new Desktop connections, workspace grants and evaluation tokens; off still allows reads, revocation and prior-account recovery |
| `ROUTING_CLERK_AUDIENCE` | unset | Expected audience of a signed Clerk session token with `sid`; required for routing tokens and Desktop bridge |
| `LINK_ROUTING_ADMISSION_EXPERIMENT` | unset | Set to `1` only with the token experiment and complete isolated service configuration |
| `LINK_ROUTING_SERVICE_KEY` | unset | Distinct 32-byte-or-longer proxy-to-Link key for internal routing actions |
| `LINK_ROUTING_PAYLOAD_MAC_KEY` | unset | Independent 32-byte hex key for account-scoped packet digests |
| `ROUTING_VERIFIED_INPUT_CEILING` | unset (closed) | Operator-verified maximum input tokens per provider call |
| `ROUTING_VERIFIED_OUTPUT_CEILING` | unset (closed) | Operator-verified maximum output tokens per provider call |
| `ROUTING_VERIFIED_TOTAL_NANO_USD_CEILING` | unset (closed) | Operator-verified all-in liability reserved per call |
| `ROUTING_RATE_CARD_REVISION` | unset (closed) | Version pinned to each operation and checked at settlement |
| `ROUTING_INPUT_NANO_USD_PER_TOKEN` | unset (closed) | Numeric input price pinned at admission |
| `ROUTING_OUTPUT_NANO_USD_PER_TOKEN` | unset (closed) | Numeric output price pinned at admission |
| `PADDLE_WEBHOOK_SECRET` | unset | Required to enable the Paddle webhook; missing secrets fail closed |
| `PADDLE_PRICE_PRO` | unset | Allowlisted Paddle price ID for Pro |
| `PADDLE_PRICE_MAX` | unset | Allowlisted Paddle price ID for Max |
| `PADDLE_PRICE_ULTRA` | unset | Allowlisted Paddle price ID for Ultra |
| `DODO_WEBHOOK_SECRET` | unset | Standard Webhooks secret; required to enable Dodo ingress |
| `DODO_BUSINESS_ID` | unset | Expected Dodo business ID |
| `DODO_BRAND_ID` | unset | Expected Pytxo brand ID |
| `DODO_PRODUCT_PRO` | unset | Allowlisted Dodo product ID for Pro |
| `DODO_PRODUCT_MAX` | unset | Allowlisted Dodo product ID for Max |
| `DODO_PRODUCT_ULTRA` | unset | Allowlisted Dodo product ID for Ultra |

Hosted production URL: `https://link.pytxo.com/v1`

Pytxo Link refuses startup when a public bind is unauthenticated or when
authentication is required without an API key or complete Clerk JWKS
configuration. Bind values must use an IP socket address such as
`127.0.0.1:8787` or `0.0.0.0:8787`.

Client env:

| Variable | Purpose |
|----------|---------|
| `PYTXO_ULTRA_SESSION` | Bearer token for reconcile + entitlements |
| `PYTXO_ORG_ID` | Optional org id for policy ceiling fetch |

`pytxo.toml` Ultra defaults (`billing.mode = "ultra"`):

```toml
[billing]
mode = "ultra"
proxy_url = "https://link.pytxo.com"
# link_reconcile defaults to true for ultra (set link_reconcile = false to disable)
```

The CLI ships with `link-http` enabled via `pytxo-orchestrate` default features.

The routing token experiment requires Postgres and Clerk JWKS/issuer in addition
to its audience and explicit flag. It accepts neither the shared `LINK_API_KEY`
nor the development auth bypass. A signed account may grant a random, opaque
workspace ID only for Link's pinned hosted recipient and scope digest. The
client must not derive that ID from a repository path. Grant/revocation uses a
monotone revision. `GET /v1/routing/workspace-grants/{workspace_id}` reads the
authenticated account's stored revision, enabled state and actual scope digest
with `Cache-Control: no-store`, so a lost enable/revoke response can be
reconciled. Desktop's current grant client pins the original scope and refuses
scope drift; changing it needs a separately reviewed grant-generation contract.
Link increments the revision on a permitted update, invalidating old tokens. Link's
scope digest is built from the fixed provider endpoint and model template,
the complete coarse task projection source, and the hosted Proxy wire handler
source. A change to those compiled sources changes the required digest. The
experimental beta retains up to 64 distinct workspace-grant rows per account,
including revoked tombstones. It has no grant deletion endpoint because
resetting a revision could revive a prior token or operation binding. A DELETE
with expected revision zero also writes a disabled revision-one tombstone when
no row exists, so a delayed first POST cannot revive an uncertain grant. This is
an explicit beta capacity limit pending a safe retirement/generation contract.
Revocation blocks later claims and answer recovery, although
an already claimed request may still send. Link stores only a hash of each
random token. Tokens are bound to one account, workspace and grant revision;
issuing another for the same grant rotates its previous token, while a second
workspace can hold its own token. Each expires after at most five minutes or
the signed session expiry. Reissue is limited to once per grant every 30 seconds
unless revoked or the grant revision changed, with ten issues per account per
rolling minute across workspaces. The separate `LINK_ROUTING_ADMISSION_EXPERIMENT=1`
gate adds internal reserve, claim, uncertain and settle operations authenticated
with a distinct `LINK_ROUTING_SERVICE_KEY`. Migrations 010–018 keep their
account, operation and global funding ledger out of subscription and Ultra
billing. Funding starts disabled with a zero ceiling. A claimed operation
permits one provider send; lost sends retain a conservative liability and a
periodic reconciler closes stale operations. Link requires a separately
verified all-in input/output/cost ceiling, pinned numeric prices and a rate
revision before admission. Those settings are not proof of the provider's
external billing bound. Desktop now offers a review-only hosted packet preview,
local consent, and a bound Link grant/revoke client behind separate experimental
switches. It fences the Catalog before remote revocation and retains uncertain
grant/revoke state for retry, including when the original workspace Store cannot
be opened. There is no network Shadow send controller, and the proxy's hard
paid-send gate remains closed. An API grant alone is not evidence that the user
reviewed a packet or authorized Live routing.

The website-to-Desktop account bridge shares the routing token experiment
gate. A signed browser session may issue at most three 60-second one-use codes
per minute. Link stores only code hashes, PKCE challenges and state hashes.
Expired codes are purged after one day and expired Desktop credential rows
after 30 days in bounded background batches, even if the experiment is off.
Desktop sends the code, state and verifier directly to Link over HTTPS; the
custom app link never contains a reusable bearer. Link rotates one scoped
Desktop credential per account, expires it after 24 hours, and stores only its
hash. An ordinary credential can read/change hosted workspace grants and request
short evaluation tokens while `LINK_ROUTING_GRANT_EXPERIMENT=1`; a recovery
credential can only read/revoke grants. Neither authenticates entitlements, Ultra or general
Link routes. Disconnect and browser lost-device revocation disable the grants
and evaluation tokens as well as the Desktop credential. Desktop checks Link's
session-status endpoint before calling a locally stored credential connected;
revoked or unreachable credentials remain visibly unverified and cannot claim
hosted readiness. Desktop defaults
to `pytxo.com` and `link.pytxo.com`. Experimental staging may set
`PYTXO_ROUTING_BRIDGE_SITE_ORIGIN` and
`PYTXO_ROUTING_BRIDGE_LINK_ORIGIN` to explicit HTTPS origins (debug builds
also permit loopback); the Link origin must match the web server's
`LINK_ROUTING_BRIDGE_URL`. Neither origin comes from a deep link. New web
account connections require `NEXT_PUBLIC_ROUTING_BRIDGE_EXPERIMENT=1`,
`LINK_ROUTING_GRANT_EXPERIMENT=1` and a
configured `LINK_ROUTING_BRIDGE_URL=https://link.pytxo.com`. Desktop additionally
requires `PYTXO_ROUTING_BRIDGE_EXPERIMENT=1` for a new connection. Creating a hosted review/local opt-in additionally requires
`PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW=1` and the configured Claude
proposal experiment; a remote grant additionally requires
`PYTXO_HOSTED_GRANT_EXPERIMENT=1`. These switches are off/unset by default.
An existing grant remains locally visible and revocable with a valid session
after the grant/review experiment switches are turned off. When Desktop's
bridge switch is also off, an unresolved grant marks its browser return with
`deck_recovery=1`; the website may issue
that one-time return while its new-connection switch is off. Link accepts a
`recovery_only` code request only for an account with prior routing activity.
Desktop rechecks all unrevoked grants after exchange and retains the credential
only when its account and Link origin match the original binding. The returned
credential has `routing:revoke:v1` scope. Link allows it to inspect/revoke
grants but rejects new grants and evaluation tokens, even if the new-grant
switch is later enabled. The proxy paid-send gate remains closed.
The website, Clerk sign-in and Link bridge must stay available for recovery
until grants are reconciled. If those services are shut down first, local sends
remain fenced but remote cleanup needs the original service restored or an
independently verified server-side revocation.
Clerk must issue its ordinary session token with an `aud` claim matching
`ROUTING_CLERK_AUDIENCE` while retaining `sid`; a custom JWT template omits
`sid` and fails Link validation. Verify the deployed claim before enabling
the bridge. An account connection is not workspace consent, a paid-send
permit or a production routing switch.

The experimental proxy requires a stable, 32-byte hex
`PROXY_ROUTING_RECOVERY_KEY` and a pinned
`PROXY_ROUTING_RECOVERY_KEY_REVISION` before a paid-send pilot. It encrypts a
minimal completed response for 24-hour recovery. A rotation must carry the
old key and revision as `PROXY_ROUTING_RECOVERY_PREVIOUS_KEY` and
`PROXY_ROUTING_RECOVERY_PREVIOUS_KEY_REVISION` for that window; stage the new
key on every replica before switching, and avoid a second rotation within
24 hours. Unavailable keys make old replies fail closed. Link
purges expired ciphertext while retaining the accounting tombstone.

The guarded PostgreSQL integration tests use a disposable loopback database
named `pytxo_routing_test` through `PYTXO_LINK_TEST_DATABASE_URL`. They return
early without that database; a green ordinary suite alone does not prove the
SQL admission transitions.

Commerce routes are unavailable without `DATABASE_URL`; in-memory commerce is
test-only because event replay protection must be durable. Both adapters feed
one transaction-backed reconciler keyed by provider and event ID. Each provider
subscription owns an independent grant, and Link projects the strongest current
grant instead of letting one provider cancel another provider's access. An
explicit admin entitlement remains an override for support and incident use.

`POST /v1/webhooks/paddle` additionally requires a non-empty
`PADDLE_WEBHOOK_SECRET` and at least one allowlisted `PADDLE_PRICE_*` value.
`subscription.created` binds Paddle subscription and customer IDs to the signed
`custom_data.user_id`; later events must match that binding.

`POST /v1/webhooks/dodo` requires all business/brand values, a Standard Webhooks
secret, and at least one `DODO_PRODUCT_*` mapping. The web proxy must preserve
the raw bytes and `webhook-id`, `webhook-timestamp`, and `webhook-signature`
headers. Link rejects stale signatures, wrong business or brand IDs, unmapped
products, conflicting principals, duplicate effects, and out-of-order upgrades.
Provider metadata never defines Pytxo capabilities.

## Local dev

```bash
cd services/pytxo-link
cargo run
```

Optional auth:

```bash
export LINK_API_KEY=dev-secret
export PYTXO_ULTRA_SESSION=dev-secret
```

Point `pytxo.toml`:

```toml
[billing]
mode = "ultra"
link_reconcile = true
proxy_url = "http://127.0.0.1:8787"
```

## Docker

```bash
docker compose up --build
```

## Production

Hosted at `https://link.pytxo.com`. Deploy steps: [`distribution/railway/README.md`](../../distribution/railway/README.md).

See [pytxo-link-service.md](../docs/08-reference/pytxo-link-service.md) for the service contract.
