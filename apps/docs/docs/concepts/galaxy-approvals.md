---
title: Galaxy approvals
---

# Galaxy approvals

On the **Galaxy** [permission tier](/docs/reference/permission-tiers), Pytxo pauses high-risk agent actions until **you** approve or deny them. That covers destructive shell commands and merging isolated copies back into your real tree.

## At a glance

- Approvals queue is **per repo** (execution domain)
- Pending requests survive restarts (`hitl.json` under `.pytxo/data/`)
- Approve from CLI, Hypervisor Shell, or Pytxo Desktop
- Orbit and lower tiers skip spawn gates; Galaxy enforces them

## What triggers approval

Examples of commands that require approval under Galaxy:

- Recursive deletes (`rm -rf …`)
- `git push`
- Docker invocations
- Network fetches (`curl`, `wget`)
- Merge / flush of agent changes into the physical repo

## CLI

List pending requests:

```bash
pytxo hitl list
```

Approve or deny:

```bash
pytxo hitl approve hitl-0
pytxo hitl deny hitl-1
```

## Hypervisor Shell

When approvals are pending, the board highlights them. With an empty prompt:

- **Tab:** cycle selection
- **a:** approve
- **x:** deny

## Pytxo Desktop

The **Approvals** panel lists pending actions with the domain they belong to. Tap Approve or Deny without switching repos manually when multiple domains have open requests.

## Troubleshooting

```bash
pytxo doctor
```

Look for the `hitl_persistence` check. It verifies the domain can write approval state to disk.

## Related

- [Permission tiers](/docs/reference/permission-tiers): when to use Galaxy vs Orbit
- [Blast Shield](/docs/concepts/blast-shield): isolated copies and approve-to-merge
- [Folder trust](/docs/getting-started/folder-trust): pick a tier when you first open a repo
