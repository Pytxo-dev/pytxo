# Pytxo on Railway — complete setup guide

Host **Pytxo Link** (`link.pytxo.com`) and **Pytxo Cloud Sandbox** (`cloud.pytxo.com`) in one Railway project with PostgreSQL. The marketing site **pytxo.com** stays on **Vercel** (auth + billing UI).

---

## Where do I find the variables?

Pytxo uses **four places** for configuration. They are **not** all in Railway.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  WHERE          │  WHAT LIVES THERE                    │  HOW TO OPEN   │
├─────────────────┼──────────────────────────────────────┼────────────────┤
│  Railway        │  Link + Cloud + Postgres secrets     │  See below     │
│  Vercel         │  Clerk keys, LINK_ADMIN_* for web    │  vercel.com    │
│  Clerk Dashboard│  Issuer / JWKS URLs for Link       │  clerk.com     │
│  Your machine   │  Local dev (.env.local)              │  apps/web      │
└─────────────────────────────────────────────────────────────────────────┘
```

### Railway — service variables (Link & Cloud)

This is what the dashboard shows when `pytxo-link` has pending **Settings** (usually missing vars or build config).

1. Open [railway.app](https://railway.app) → project **<your-project>** (name is random; yours may differ).
2. **Click the `pytxo-link` card** on the canvas (not Postgres).
3. A panel opens on the right (or full-page view). Use the tabs at the top:
   - **Variables** ← add `LINK_API_KEY`, `DATABASE_URL`, etc.
   - **Settings** ← Dockerfile path, root directory, domains
   - **Deployments** ← build logs
   - **Logs** ← runtime errors after deploy

**To add a variable:**

1. `pytxo-link` → **Variables** tab.
2. Click **+ New Variable** (or **Raw Editor** to paste many at once).
3. For Postgres, use **Add Reference** (not a pasted URL):
   - Variable name: `DATABASE_URL`
   - Click **Add Reference** → select **Postgres** → `DATABASE_URL`
   - Railway stores it as `${{Postgres.DATABASE_URL}}` in the UI.

**To view Postgres-only variables** (rarely needed):

1. Click the **Postgres** card → **Variables** tab.
2. You’ll see `DATABASE_URL`, `PGHOST`, `PGUSER`, etc. Link should **reference** these, not copy them.

**CLI equivalent** (from repo root, after `railway link -p <your-project>`):

```powershell
railway variables -s pytxo-link          # list Link vars
railway variables -s Postgres            # list DB vars
railway variables set LINK_REQUIRE_AUTH=1 -s pytxo-link
```

### Vercel — website + billing bridge

Sign-in keys and the webhook → Link admin bridge live on **Vercel**, not Railway.

1. [vercel.com](https://vercel.com) → your **pytxo** project → **Settings** → **Environment Variables**.

| Variable | Purpose |
|----------|---------|
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | Sign-in / sign-up on pytxo.com |
| `CLERK_SECRET_KEY` | Server-side Clerk on pytxo.com |
| `LINK_ADMIN_URL` | `https://link.pytxo.com` |
| `LINK_ADMIN_KEY` | Must match Railway `LINK_ADMIN_KEY` |

Pull locally: `cd apps/web && vercel env pull`  
Push from Clerk: `clerk env pull` then add to Vercel (already done if you deployed recently).

### Clerk Dashboard — URLs for Railway Link only

Link verifies JWTs for org-policy routes. You need **issuer** and **JWKS** URLs (not the secret key on Link).

1. [dashboard.clerk.com](https://dashboard.clerk.com) → application **pytxo**.
2. **Configure** → **API Keys** (or **JWT templates** area).
3. Copy:
   - **Issuer** → Railway `CLERK_ISSUER`  
     Example shape: `https://<instance>.clerk.accounts.dev`
   - **JWKS URL** → Railway `CLERK_JWKS_URL`  
     Example: `https://<instance>.clerk.accounts.dev/.well-known/jwks.json`

Or from your machine after `clerk env pull`:

```powershell
cd C:\pytxo\apps\web
clerk env pull
# CLERK_ISSUER / CLERK_JWKS_URL may appear in .env.local — copy values into Railway Link Variables
```

### Generated secrets (you create these)

Railway does **not** generate `LINK_API_KEY` / `LINK_ADMIN_KEY` / `CLOUD_API_KEY`. Generate once:

```powershell
cd C:\pytxo
.\tooling\scripts\deploy-railway.ps1
```

Save the three printed keys. Use:

- `LINK_API_KEY` + `LINK_ADMIN_KEY` → Railway **pytxo-link** → Variables
- `LINK_ADMIN_KEY` → Vercel (same value)
- `CLOUD_API_KEY` → Railway **pytxo-cloud-sandbox** → Variables (when you add that service)

---

## Your current dashboard

If you see **Postgres (Online)** and **pytxo-link (New)** with **N Settings** and **Apply N changes** at the top:

| Card | Status | Next step |
|------|--------|-----------|
| **Postgres** | Online | Nothing — ready |
| **pytxo-link** | New, pending settings | Finish Settings + Variables, then **Apply N changes** → **Deploy** |

**“N Settings”** on the card = incomplete **Settings** (Dockerfile path, variables, networking, etc.).

**“Apply N changes”** = staged edits not live until you **Apply** → **Deploy**.

1. **Apply N changes** (review in Details if you want).
2. **Deploy** (purple button, or `Ctrl+Enter`).

Until you deploy, variables and Dockerfile paths are **not** running in production.

---

## What you are building

```
pytxo.com (Vercel)
  ├── Clerk sign-in / sign-up
  └── Paddle webhook → PUT /v1/admin/entitlements → link.pytxo.com (Railway)

pytxo CLI / Deck
  ├── link.pytxo.com  (billing, entitlements, runs)
  └── cloud.pytxo.com (Max-tier cloud sandbox)

link.pytxo.com → Postgres (Railway plugin)
```

| Resource | Where | Public URL |
|----------|--------|------------|
| Postgres | Railway plugin | internal only |
| `pytxo-link` | Railway Docker service | `link.pytxo.com` |
| `pytxo-cloud-sandbox` | Railway Docker service | `cloud.pytxo.com` |
| Web + auth | Vercel | `pytxo.com` |

---

## Part 0 — Prerequisites

1. **Railway** account with an active plan ([Billing](https://railway.app/account/billing) if trial expired).
2. **GitHub** repo `Pytxo-dev/pytxo` connected to Railway.
3. **Vercel** project for `pytxo.com` with Clerk env vars.
4. **Clerk** application for issuer/JWKS URLs (Link service).
5. **DNS** access for `link.pytxo.com` and `cloud.pytxo.com` (CNAME to Railway).

**CLI (optional):**

```powershell
npm i -g @railway/cli
railway login
cd C:\pytxo
railway link -p <your-project>
```

Project names are random (`<your-project>`); only service names should be `pytxo-link` and `pytxo-cloud-sandbox`.

---

## Part 1 — pytxo-link (you are here)

### Step 1: Postgres

You already have **Postgres → Online**. If not: **+ Add** → **Database** → **PostgreSQL**.

### Step 2: Rename the GitHub service

If the card isn’t already named `pytxo-link`:

1. Click the service card → **Settings**.
2. **Service name** → `pytxo-link` → save.

### Step 3: Configure build settings (critical)

`pytxo-link` → **Settings** → **Build**:

| Setting | Value |
|---------|--------|
| **Root Directory** | empty or `/` (repo root) |
| **Dockerfile Path** | `services/pytxo-link/Dockerfile` |
| **Config-as-code file** | `services/pytxo-link/railway.toml` |

| Setting | Do NOT set |
|---------|------------|
| Custom start command | leave empty |
| `PORT` / `LINK_BIND` in Variables | Railway sets `PORT` automatically |

### Step 4: Generate API keys

```powershell
cd C:\pytxo
.\tooling\scripts\deploy-railway.ps1
```

Press `N` to skip deploy — copy the three keys somewhere safe.

### Step 5: Environment variables for `pytxo-link`

`pytxo-link` → **Variables** → add each row:

| Variable | How to set | Example / notes |
|----------|------------|-------------------|
| `DATABASE_URL` | **Add Reference** → Postgres → `DATABASE_URL` | `${{Postgres.DATABASE_URL}}` |
| `LINK_REQUIRE_AUTH` | Raw value | `1` |
| `LINK_API_KEY` | Raw value | from deploy script |
| `LINK_ADMIN_KEY` | Raw value | from deploy script (same on Vercel) |
| `CLERK_ISSUER` | Raw value | from Clerk Dashboard |
| `CLERK_JWKS_URL` | Raw value | from Clerk Dashboard |

**Optional:**

| Variable | When |
|----------|------|
| `PADDLE_WEBHOOK_SECRET` | Link verifies Paddle directly |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Tracing |
| `SENTRY_DSN` | Error reporting |

**Raw Editor example** (replace secrets with your values):

```env
DATABASE_URL=${{Postgres.DATABASE_URL}}
LINK_REQUIRE_AUTH=1
LINK_API_KEY=<paste>
LINK_ADMIN_KEY=<paste>
CLERK_ISSUER=https://<your-instance>.clerk.accounts.dev
CLERK_JWKS_URL=https://<your-instance>.clerk.accounts.dev/.well-known/jwks.json
```

### Step 6: Deploy Link

1. Top bar → **Apply N changes** → **Deploy**.
2. `pytxo-link` → **Deployments** → open latest → **Build Logs** (Rust compile: 5–15 min first time).
3. When done: **Settings** → **Networking** → copy the `*.up.railway.app` URL.
4. Test: `https://<that-url>/health` → should return OK.

### Step 7: Custom domain `link.pytxo.com`

1. `pytxo-link` → **Settings** → **Networking** → **+ Custom Domain**.
2. Enter `link.pytxo.com`.
3. Add **CNAME** at your DNS host:

   | Type | Name | Target |
   |------|------|--------|
   | CNAME | `link` | value Railway shows |

4. Wait for TLS → domain shows **Active**.

---

## Part 2 — pytxo-cloud-sandbox

Add after Link is healthy.

1. **+ Add** → **GitHub Repo** → `Pytxo-dev/pytxo`.
2. Rename service → **`pytxo-cloud-sandbox`**.
3. **Settings** → Build:

   | Setting | Value |
   |---------|--------|
   | Root Directory | `/` |
   | Dockerfile Path | `services/pytxo-cloud-sandbox/Dockerfile` |
   | Config file | `services/pytxo-cloud-sandbox/railway.toml` |

4. **Variables** tab:

   | Variable | Value |
   |----------|--------|
   | `CLOUD_API_KEY` | from deploy script |

5. **Apply changes** → **Deploy**.
6. **Networking** → custom domain `cloud.pytxo.com` → CNAME at DNS.

Verify:

```powershell
Invoke-WebRequest https://link.pytxo.com/health
Invoke-WebRequest https://cloud.pytxo.com/health
Invoke-WebRequest https://link.pytxo.com/openapi.json
```

---

## Part 3 — Wire Vercel to Link

your Vercel project → **Settings** → **Environment Variables**

| Variable | Value |
|----------|--------|
| `LINK_ADMIN_URL` | `https://link.pytxo.com` |
| `LINK_ADMIN_KEY` | **exact same** string as Railway `LINK_ADMIN_KEY` |

Redeploy Vercel after changes (`vercel deploy --prod` from `apps/web`).

**Billing flow test:**

1. Paddle → `https://pytxo.com/api/billing/paddle/webhook`
2. Vercel → `PUT https://link.pytxo.com/v1/admin/entitlements/{user_id}`
3. On failure: `pytxo-link` → **Logs** on Railway.

---

## Part 4 — Full variable cheat sheet

### Railway `pytxo-link`

| Variable | Required | Source |
|----------|----------|--------|
| `DATABASE_URL` | yes | Reference → Postgres |
| `LINK_REQUIRE_AUTH` | yes | `1` |
| `LINK_API_KEY` | yes | deploy script |
| `LINK_ADMIN_KEY` | yes | deploy script + Vercel |
| `CLERK_ISSUER` | yes | Clerk Dashboard |
| `CLERK_JWKS_URL` | yes | Clerk Dashboard |
| `PORT` | auto | Railway injects — do not set |
| `PADDLE_WEBHOOK_SECRET` | optional | Paddle |

### Railway `pytxo-cloud-sandbox`

| Variable | Required | Source |
|----------|----------|--------|
| `CLOUD_API_KEY` | yes | deploy script |
| `PORT` | auto | Railway injects |

### Vercel (pytxo.com)

| Variable | Required | Source |
|----------|----------|--------|
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | yes | `clerk env pull` |
| `CLERK_SECRET_KEY` | yes | `clerk env pull` |
| `LINK_ADMIN_URL` | yes | `https://link.pytxo.com` |
| `LINK_ADMIN_KEY` | yes | same as Railway |
| `MBCZ_*`, `PADDLE_*` | billing | MBCZ / Paddle setup |

### Local dev (`apps/web/.env.local`)

```powershell
cd C:\pytxo\apps\web
clerk env pull
pnpm dev
```

Link/Cloud locally: see root [`.env.example`](../../.env.example).

---

## Part 5 — CLI reference

```powershell
cd C:\pytxo
railway link -p <your-project>
railway status
railway open

railway variables -s pytxo-link
railway logs -s pytxo-link
railway up -s pytxo-link

railway link -s pytxo-link    # if Service: None
```

---

## Part 6 — Checklist

```
Railway (<your-project>)
[ ] Postgres Online
[ ] pytxo-link Settings: Dockerfile + railway.toml
[ ] pytxo-link Variables: DATABASE_URL reference + LINK_* + CLERK_*
[ ] Apply changes → Deploy → /health OK
[ ] CNAME link.pytxo.com
[ ] pytxo-cloud-sandbox (optional second phase)
[ ] CNAME cloud.pytxo.com

Vercel
[ ] Clerk keys on Production
[ ] LINK_ADMIN_URL + LINK_ADMIN_KEY
[ ] pytxo.com sign-in works

End-to-end
[ ] curl link.pytxo.com/health
[ ] Sign up on pytxo.com → account page
[ ] Paddle webhook → entitlement on Link (when billing configured)
```

---

## Troubleshooting

### “N Settings” on pytxo-link card

Open **Settings** — usually missing Dockerfile path, config file, or required variables. Fix, then **Apply** → **Deploy**.

### “Apply N changes” but service won’t start

Variables and Dockerfile changes are staged until you click **Deploy**. Check **Deployments** → failed build → **Build Logs**.

### Build fails on `cargo build`

- Root directory must be repo root, not `services/pytxo-link`.
- Dockerfile: `services/pytxo-link/Dockerfile`.

### Crash on startup / health check fails

- `pytxo-link` → **Logs**: look for `DATABASE_URL` or migration errors.
- Use **Reference** for `DATABASE_URL`, not a localhost URL.

### 401 from Link API

- Run routes: `Authorization: Bearer <LINK_API_KEY>`.
- Admin routes: `LINK_ADMIN_KEY`.

### Sign-in works on pytxo.com but Link rejects JWT

- `CLERK_ISSUER` and `CLERK_JWKS_URL` on Railway must match the Clerk instance that issued the user’s session.

### Wrong Railway project in CLI

```powershell
railway unlink --yes
railway link -p <your-project>
```

---

## Local development

```bash
cd services/pytxo-link && cargo run          # 127.0.0.1:8787
cd services/pytxo-cloud-sandbox && cargo run # 127.0.0.1:8788
```

```powershell
$env:PORT=3000; cargo run -p pytxo-link   # tests Railway PORT binding
```

---

## Repo reference

| File | Purpose |
|------|---------|
| `services/pytxo-link/railway.toml` | Build + health check |
| `services/pytxo-link/railway.json` | Env var list (machine-readable) |
| `services/pytxo-cloud-sandbox/railway.toml` | Cloud build config |
| `tooling/scripts/deploy-railway.ps1` | Generate API keys |
| `apps/web/.env.local.example` | Clerk local setup |
| `.env.example` | Link/Cloud local template |
