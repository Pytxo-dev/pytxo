# Pytxo marketing site

Next.js site for [pytxo.com](https://pytxo.com). Public docs are Fumadocs MDX under `content/docs` and render at `/docs` in the same app.

## Development

```bash
pnpm install
pnpm dev
```

Open [http://localhost:3000/docs](http://localhost:3000/docs) for documentation.

## Production build

```bash
pnpm build
```

## Vercel

Set the Vercel project **Root Directory** to `apps/web`.

```bash
pnpm deploy   # vercel deploy --prod
```

Custom domain: **pytxo.com**
