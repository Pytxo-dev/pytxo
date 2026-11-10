# Pytxo

**Clear your agent PR pile in one command.**

Pytxo merges your Claude Code, Codex and Copilot branches in a throwaway
worktree, runs your tests on the combined result, and lands the ones that
pass. Every branch it leaves out gets the reason and a fix prompt to hand back
to its agent. Your own tests decide; Pytxo calls no AI model. Free, open
source, runs on your machine or in CI.

```bash
npx pytxo demo      # a throwaway repo: four agent branches, two land, two get the reason
cd your-repo
git fetch           # get the branches your agents pushed
npx pytxo           # checks your agent branches together
npx pytxo --apply   # lands the ones that pass
```

This repository holds the release binaries and install scripts. The source is
[Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) (MIT).

[Docs](https://pytxo.com/docs) · [Source](https://github.com/Pytxo-dev/pytxo) ·
[Discord](https://discord.gg/AUFRPFjSYv) · [pytxo.com](https://pytxo.com)

## Install

**npx (Node.js 18 or newer):** nothing to install. The first `npx pytxo`
downloads the binary for your platform from this repository's releases and
checks it against `SHA256SUMS.txt`.

**npm, to keep it on your PATH:**

```bash
npm i -g pytxo
pytxo --version
```

npm 12 blocks install scripts by default and prints a warning about pytxo's
`postinstall`; you can ignore it.

**Install script (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

**Install script (Windows PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.ps1 | iex
```

Each release lists its assets, SHA-256 checksums (`SHA256SUMS.txt`), and
platform and signing notes. Read them before installing.

**Pytxo Desktop (Windows x64)** is frozen: no new features, and still a free
download as `pytxo-desktop-windows-x64.msi` from the
[latest release](https://github.com/Pytxo-dev/pytxo-releases/releases/latest).

## Limits

- It catches what your tests catch. A break your tests don't cover can still
  land, and merge conflicts are blocked, not resolved.
- The check runs your own test command in a fresh worktree, so it is as slow
  as your tests plus installing dependencies.
- Pytxo calls no model and sends nothing anywhere.

## Feedback

Bugs and questions: [open an issue](https://github.com/Pytxo-dev/pytxo-releases/issues)
or ask in [Discord](https://discord.gg/AUFRPFjSYv). Include the Pytxo version,
the verdict you got, and what you expected.
