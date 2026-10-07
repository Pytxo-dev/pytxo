# Pytxo

**Many coding agents. One verified change.**

Describe a change once. Pytxo splits it across the coding-agent CLIs you already
use (Codex, Claude Code, Cursor Agent, OpenCode, Antigravity), runs each task
in its own isolated copy of your project, keeps tasks that share a file from
running together, runs your checks on the combined result, and applies exactly
the files you reviewed.

![One request split across five agents in Pytxo Desktop](https://pytxo.com/product/fleet-1600x1000.png)

[Download](https://pytxo.com/download) · [Docs](https://pytxo.com/docs) ·
[Source (MIT)](https://github.com/Pytxo-dev/pytxo) ·
[Discord](https://discord.gg/AUFRPFjSYv) · [pytxo.com](https://pytxo.com)

## Why

Running several agents at once is easy now. Ending up with one change you can
trust is not: ten agents usually means ten branches to read and merge. Pytxo
works on the end of the run.

- **One plan, split by file ownership.** Each task owns the files it may
  change. Write one task per line, or let Codex or Claude Code read your project
  in its read-only mode and propose the split. Edit any step before it runs.
- **Isolated work.** Every task runs in its own copy of the project with the
  agent you assigned. Your repository is untouched until you apply.
- **Checks Pytxo runs itself.** An agent saying "done" is not a pass. Pytxo
  runs your commands on each task and again on all changes together.
- **Exact Apply.** Review every changed line and which agent wrote it. Apply
  writes those exact bytes, refuses if your project changed since review, and
  journals the attempt so an interrupted Apply is reconciled on restart.

Pytxo is a control layer around agents, not another agent or IDE. It uses each
agent's own sign-in. There is no Pytxo account for local work, and agents run
with a clean environment, so API keys set in your shell are not passed to them.

## Install

**Pytxo Desktop (Windows x64):** download `pytxo-desktop-windows-x64.msi` from
the [latest release](https://github.com/Pytxo-dev/pytxo-releases/releases/latest)
or [pytxo.com/download](https://pytxo.com/download). Desktop includes its local
core; the CLI below is optional. macOS and Linux Desktop builds are not
available yet.

You need at least one supported agent CLI installed and signed in. Desktop
detects them on first run and tells you what is missing.

**CLI (npm):**

```bash
npm i -g pytxo
pytxo doctor
```

**CLI install script (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

**CLI install script (Windows PowerShell):**

```powershell
irm https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.ps1 | iex
```

Each release lists its assets, SHA-256 checksums (`SHA256SUMS.txt`), and
platform and signing notes. Read them before installing.

## Limits

- Pytxo controls what reaches your repository. It does not sandbox every file
  or network action an agent takes on your machine; each run lists which
  protections were enforced and which were advisory.
- Passing checks means your commands passed on the exact files you are about
  to apply, not that the code is right in every way.
- Splitting helps when a job has parts that can run at once. A small fix is
  often quicker with one agent, and Pytxo works fine with one.

## Feedback

Bugs and questions: [open an issue](https://github.com/Pytxo-dev/pytxo-releases/issues)
or ask in [Discord](https://discord.gg/AUFRPFjSYv). Include the Pytxo version,
the agent CLI and its version, and what you expected to happen.
