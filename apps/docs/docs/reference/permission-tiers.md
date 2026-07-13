---
title: Permission tiers
---

# Permission tiers

Pytxo uses local **permission profiles** to gate filesystem, network, and tool capabilities per agent. Set globally, per `[[agent]]`, or per [modular project](/docs/concepts/modular-projects) root.

| Profile | Intent |
|---------|--------|
| `deep_space` | Most restrictive: read-heavy, minimal side effects |
| `orbit` | **Default:** balanced local development with isolated copies |
| `galaxy` | Host tools plus **[human approval](/docs/concepts/galaxy-approvals)** for risky commands and merges |
| `supernova` | Highest local capability (use deliberately) |

```toml
permission_profile = "orbit"

[[agent]]
name = "scout"
permission_profile = "deep_space"
```

## Galaxy in practice

When you trust a folder as **Galaxy**:

- Risky spawn commands (`rm -rf`, `git push`, `docker`, network tools) pause until you approve
- Merging agent changes into your real tree requires approval
- Pending requests persist across restarts

Use `pytxo hitl list` or the Desktop Approvals panel to respond.

## Folder trust

The first time you open a repo in the Hypervisor Shell, you pick a tier. See [Folder trust](/docs/getting-started/folder-trust).

Permission profiles are **not** the same as `signal_fidelity` (context compression tier).

Ultra-tier managed metering (`[billing]` with `mode = "ultra"`) is a separate commercial layer. BYOK remains the default.
