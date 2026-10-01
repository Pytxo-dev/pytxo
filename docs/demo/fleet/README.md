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
- OpenCode needs a model that supports tool calls. The fixture's
  `opencode.json` pins `openrouter/~moonshotai/kimi-latest`; change it if your
  OpenCode account uses another provider.

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

Record the Desktop window at 3840×2160 from step 5 to step 8. Keep the run's
History entry: the film's on-screen numbers (workers, waves, checks, files and
package digest) must come from that recorded run.
