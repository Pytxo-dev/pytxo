# Maintaining pytxo-releases

Private note; the release workflow syncs only `README.md`, `install.sh`,
`install.ps1` and `latest.json` to the public repository.

`Pytxo-dev/pytxo-releases` is the public distribution channel: CLI binaries,
Desktop installers, the signed updater manifest (`latest.json`), install
scripts and `SHA256SUMS.txt`. Releases before v0.5.0 used the legacy
`pytxo-reality-deck-*` asset prefix. Compatibility mirrors keep their original
embedded version and are not evidence of a fresh platform build.

The public README loads its screenshot from `pytxo.com`, so deploy the website
before the release sync publishes a README that references new captures.

## Setup

1. In the private `pytxo` repository, add the secret `PYTXO_RELEASES_TOKEN`
   (PAT or GitHub App) with `contents: write` on `pytxo-releases`.
2. Tag a release in `pytxo` (`git tag vX.Y.Z && git push origin vX.Y.Z`) or run
   **Actions → Release / Desktop release**. CI mirrors the CLI and Desktop
   installers to this repository's GitHub Release and syncs the files above.

Pin a CLI version with the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | PYTXO_VERSION=v1.2.2 bash
```
