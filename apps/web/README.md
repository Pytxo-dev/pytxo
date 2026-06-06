# Pytxo marketing site

Next.js static marketing site for [pytxo.com](https://pytxo.com). Public docs are built with Docusaurus in `../docs` and copied to `public/docs/` at build time.

## Development

```bash
npm install --legacy-peer-deps
npm run dev
```

For docs preview during development:

```bash
cd ../docs && npm install && npm start   # localhost:3000/docs/
# or after a full docs build:
cd ../web && npm run build:docs && npm run copy:docs
```

## Production build

```bash
npm run build   # builds Docusaurus, copies to public/docs, static Next export → out/
```

## Vercel

Set the Vercel project **Root Directory** to `apps/web`.

```bash
npm run deploy   # vercel deploy --prod
```

Custom domain: **pytxo.com**
