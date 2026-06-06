---
title: Permission tiers
---

# Permission tiers

Pytxo uses local **permission profiles** to gate filesystem, network, and tool capabilities per agent. Set globally or per `[[agent]]`.

| Profile | Intent |
|---------|--------|
| `deep_space` | Most restrictive — read-heavy, minimal side effects |
| `orbit` | **Default** — balanced local development |
| `galaxy` | Broader write and network scope |
| `supernova` | Highest local capability (use deliberately) |

```toml
permission_profile = "orbit"

[[agent]]
name = "scout"
permission_profile = "deep_space"
```

Permission profiles are **not** the same as `signal_fidelity` (context compression tier).

Ultra-tier managed metering (`[billing]` with `mode = "ultra"`) is a separate commercial layer — BYOK remains the default.
