# Pytxo marketing site

Next.js site for [pytxo.com](https://pytxo.com). Public docs are Fumadocs MDX under `content/docs` and render at `/docs` in the same app.

## Development

```bash
npm ci --legacy-peer-deps
npm run dev
```

Open [http://localhost:3000/docs](http://localhost:3000/docs) for documentation.

## Production build

```bash
npm run build
```

The build fails if Fumadocs loads fewer than 20 MDX pages.

## Vercel

Canonical project: `prj_2741otn0udtebonWb6wVx7EH8UiO` (team MBCZ).

- Git-link **Pytxo-dev/pytxo**, root directory `apps/web`, framework **Next.js**.
- Production aliases (`pytxo.com`) only from **`main`**. Do not promote feature-branch CLI deploys.
- Required GitHub secrets for `.github/workflows/deploy-web.yml`: `VERCEL_TOKEN`, `VERCEL_ORG_ID` (`team_9sY8Z8TDoLtbWEte8M95C2mY`), `VERCEL_PROJECT_ID` (`prj_2741otn0udtebonWb6wVx7EH8UiO`).

Custom domain: **pytxo.com**
