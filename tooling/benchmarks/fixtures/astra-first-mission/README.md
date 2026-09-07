# Three-task first mission

This six-file repository is the baseline for the September 7 Codex observations.
`fixture/` contains only the files the agent should see. The mission and baseline
hashes sit alongside it as reproduction metadata. Do not copy this parent folder
into the working repository or reuse a previously modified fixture.

Use an existing, authenticated Codex CLI and an approved local Pytxo candidate.
The recorded runs used Codex 0.153.4 with its existing user configuration. Model
settings, startup hooks and background machine load affect the observations.
This fixture installs no dependencies and calls no network services in its tests.

Create a fresh directory outside the Pytxo source repository. Copy the six files
from `fixture/`, including `.gitignore`, and initialize a local Git baseline:

```powershell
git init
git config --local core.autocrlf false
git config --local user.name "Pytxo validation"
git config --local user.email "validation@example.invalid"
git add -- .gitignore README.md package.json pytxo.toml src test
git commit -m "Prepare three-task mission baseline"
npm test
```

The author settings apply only to this disposable fixture. The baseline has two
passing tests. Verify each file against
`BASELINE-SHA256.json` before running. The repository configuration sets Orbit,
worktree isolation, PTY execution, fail-fast behavior and at most two concurrent
workers. The runtime receipt, rather than this configuration alone, establishes
which isolation mechanism was actually used.

Open this folder in Desktop, choose Codex, paste the exact contents of
`MISSION.txt`, and select **Build plan**. Inspect three tasks: implementation and
documentation in the first wave, followed by tests depending on implementation.
Each task owns one file and records `npm test`. For the final rehearsal, use the
editable documentation-task field to replace its prompt with the exact contents
of `DOCUMENTATION-TASK.txt`, then review before Run. This clarifies the requested
combined behavior: an earlier ready package passed checks but review withheld
Apply because its independently written README described the old implementation.
The original mission text remains unchanged; this task-specific override is
additional guidance and must be disclosed in comparisons with the direct run.

Keep all outcomes, including refused or failed runs. Before Apply, compare all
six primary-file hashes with the baseline and check that no source files were
added or removed, excluding Git and Pytxo runtime metadata. Inspect every exact diff, the combined
candidate checks and the enforcement receipt in native Review. Confirm the exact
package only if its contents meet the mission. After Apply, compare each changed
file with its frozen package digest, check unrelated baseline files and rerun
`npm test`. Record the Apply journal and the exact executable/installer hashes.

For a direct comparison, create a separate Git worktree from the same six-file
baseline and provide the same mission to the same Codex CLI. Independently inspect
its diff and run its tests. A direct Git worktree also preserves the primary
checkout; that property alone is not evidence of a Pytxo advantage. Process exit
and combined-candidate verification are different timing endpoints. Report model
usage when available; leave unobserved cost and operator time unset.

The result records in `../../results/` are individual observations, not a
reliability estimate, a controlled speed comparison or clean-install evidence.
