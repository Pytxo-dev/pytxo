# pytxo-releases

**Public distribution channel** for [Pytxo](https://pytxo.com) CLI and **Pytxo Desktop** installers.

The main `Pytxo-dev/pytxo` monorepo is private. This repository is **public** and hosts:

- Prebuilt CLI binaries (`pytxo-linux-x64`, `pytxo-darwin-arm64`, …)
- Pytxo Desktop installers (`pytxo-reality-deck-*.msi` / `.dmg` / `.AppImage` — legacy asset prefix; product name is Pytxo Desktop)
- Install scripts (`install.sh`, `install.ps1`) for the CLI
- Checksums (`SHA256SUMS.txt`)

## Install (users)

**Desktop (recommended for operators):** [pytxo.com/download](https://pytxo.com/download)

| Platform | Asset |
|----------|--------|
| Windows x64 | `pytxo-reality-deck-windows-x64.msi` |
| macOS Apple Silicon | `pytxo-reality-deck-darwin-arm64.dmg` |
| macOS Intel | `pytxo-reality-deck-darwin-x64.dmg` |
| Linux x64 | `pytxo-reality-deck-linux-x64.AppImage` |

**CLI via npm:**

```bash
npm i -g pytxo
pytxo doctor
```

**CLI install script (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

**Windows CLI (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.ps1 | iex
```

Pin a CLI version:

```bash
PYTXO_VERSION=v0.4.0 curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
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

4. Tag a release in `pytxo` (`git tag v0.4.0 && git push origin v0.4.0`), or run **Actions → Release / Desktop release**. CI mirrors CLI + Desktop installers to this repo’s GitHub Release.

## Layout

| Path | Purpose |
|------|---------|
| `install.sh` | macOS / Linux CLI installer |
| `install.ps1` | Windows CLI installer |
| Releases | Tagged `v*` with CLI binaries + Desktop installers |
