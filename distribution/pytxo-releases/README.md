# pytxo-releases

**Public distribution channel** for the [Pytxo](https://pytxo.com) CLI.

The main `Pytxo-dev/pytxo` monorepo is private. This repository is **public** and hosts:

- Prebuilt release binaries (`pytxo-linux-x64`, `pytxo-darwin-arm64`, …)
- Install scripts (`install.sh`, `install.ps1`)
- Checksums (`SHA256SUMS.txt`)

## Install (users)

**npm (recommended):**

```bash
npm i -g pytxo
pytxo doctor
```

**Install script (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.ps1 | iex
```

Pin a version:

```bash
PYTXO_VERSION=v0.3.0 curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

## Maintainer setup

1. Create a **public** GitHub repo: `Pytxo-dev/pytxo-releases`.
2. Push this directory as `main`:

   ```bash
   cd distribution/pytxo-releases
   git init
   git remote add origin git@github.com:Pytxo-dev/pytxo-releases.git
   git add .
   git commit -m "Initial public distribution channel"
   git push -u origin main
   ```

3. In the private `pytxo` repo, add GitHub secret `PYTXO_RELEASES_TOKEN` (PAT or GitHub App) with `contents: write` on **pytxo-releases**.

4. Tag a release in `pytxo` (`git tag v0.3.0 && git push origin v0.3.0`). CI mirrors binaries to this repo’s GitHub Release.

## Layout

| Path | Purpose |
|------|---------|
| `install.sh` | macOS / Linux installer |
| `install.ps1` | Windows installer |
| Releases | Tagged `v*` with platform binaries |
