# Fleet demo: six agents, five vendors, one repository

A reproducible Desktop run for the fleet film and native acceptance. A small
task board gets six tasks from one request. Pytxo plans them in three waves and
assigns them across five CLIs:

| Wave | Task | Owns | CLI |
| --- | --- | --- | --- |
| 1 | Search and status filtering, with tests | `src/model.mjs`, `test/model.test.mjs` | OpenAI Codex |
| 1 | Dark theme and theme toggle | `src/style.css`, `src/theme.js` | Claude Code |
| 1 | Spanish strings and language switch | `src/i18n/es.json`, `index.html` | Cursor Agent |
| 1 | Filter bar component | `src/components/filter-bar.js` | OpenCode |
| 2 | Blank-title validation; mounts the filter bar | `src/app.js`, `src/components/filter-bar.js`, `test/validation.test.mjs` | Antigravity |
| 3 | README for the combined result | `README.md` | OpenAI Codex |

Wave 2 waits because it shares the filter bar with OpenCode and its
`src/app.js` imports the model Codex is changing. Wave 3 documents the result
of every earlier task. `cargo test -p pytxo-orchestrate --test flow fleet_demo`
pins this plan shape, so a planner change that alters it fails in CI.

## Prerequisites

- Windows x64 with Git and Node.js on `PATH`.
- Pytxo Desktop from the release candidate MSI.
- Signed in and on `PATH`: `codex`, `claude`, `cursor-agent`, `opencode`, `agy`.
- OpenCode needs a model that supports tool calls, from a provider stored with
  `opencode auth login`. Pytxo does not pass environment API keys such as
  `OPENROUTER_API_KEY` to workers, so a key that exists only in your
  environment is not enough. The fixture's `opencode.json` pins
  `openrouter/~moonshotai/kimi-latest`; change it to match your stored
  provider.

Each CLI runs on your own account and may consume its quota.

## Run it

1. Create the fixture repository:

   ```powershell
   ./docs/demo/fleet/setup.ps1
   ```

   It creates `%USERPROFILE%\pytxo-demo\taskboard` as a fresh Git repository.
   Pass `-Path` to use another folder.
2. In Pytxo Desktop, add that folder from the workspace switcher and trust it at
   Orbit.
3. Open **New work** and paste the contents of `mission.txt`.
4. Set **Agent CLI** to OpenAI Codex. Under **Also put to work**, select Claude
   Code, Cursor Agent, OpenCode and Antigravity in that order. Tasks are
   assigned in that order, which produces the table above.
5. **Build plan** and confirm the per-task agents match the table above, then
   **Run**.
6. Follow the run on the fleet board in **Work**. When it finishes, open
   **Review changes**: each file names the CLI that prepared it.
7. Optional stale check: add any file to the repository, return to Review and
   confirm Apply is refused until you refresh. Remove the file and refresh.
8. **Apply reviewed changes**, then run `npm test` in the fixture.

## Reset

```powershell
./docs/demo/fleet/reset.ps1
```

It deletes the fixture only if `setup.ps1` created it.

## Film capture

`capture/` reproduces the run for the film without manual steps:

```powershell
./docs/demo/fleet/setup.ps1 -Path D:/fleet-demo/taskboard
./docs/demo/fleet/capture/launch.ps1 -Msi <candidate MSI> -Root D:/fleet-demo/evidence
node docs/demo/fleet/capture/capture.mjs --root D:/fleet-demo/evidence --repo D:/fleet-demo/taskboard
```

`launch.ps1` extracts the MSI without installing it and starts Desktop with
isolated Pytxo state and a local CDP port. `capture.mjs` drives onboarding (the
folder is chosen through Desktop's real folder dialog by `pick-folder.ps1`),
the request, plan, run, Review, the stale refusal and Apply, while recording
the app's own frames through CDP screencast into `capture.mp4`. It writes
`clicks.json` (each click's target and time, for a pointer drawn in the edit;
the OS pointer is not recorded) and screenshots of each beat. Then export the
ledger for the film with `apps/demo-video`'s `npm run fleet:ledger`. Pass
`--until plan` to stop before any agent runs.
