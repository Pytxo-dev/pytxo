# Pytxo public docs

Docusaurus site served at [pytxo.com/docs/](https://pytxo.com/docs/). Curated consumer and developer documentation for installers and contributors.

## Development

```bash
npm install
npm start
```

Open `http://localhost:3000/docs/` (baseUrl is `/docs/`).

## Production

Built automatically during `apps/web` prebuild:

```bash
npm run build
```

Output is copied to `apps/web/public/docs/` before Next.js static export.
