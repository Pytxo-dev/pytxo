---
title: Competitive landscape July 2026
slug: competitive-landscape-2026-07
status: active
tags: [project, research, competitive, positioning, gtm]
audience: [human, agent]
layer: meta
created: 2026-07-26
updated: 2026-08-27
related: [[multi-agent-orchestration-landscape]], [[product-vision]], [[commit-layer]], [[pytxo-commit-layer-alignment]], [[beyond-the-ade]], [[pytxo-vs-claude-agent-teams]], [[pytxo-vs-github-copilot-app]], [[pytxo-vs-ade-virtual-workspace]], [[market-ready-polish-research]], [[competitive-benchmarks]], [[pytxo-improvement-research]]
---

# Competitive landscape — July 2026

> [!NOTE]
> Dated market snapshot. Its comparison data remains useful, but the local
> hypervisor category recommendation was superseded by the
> [[product-vision|commit-layer vision]]. Phase 0 of
> [[pytxo-commit-layer-alignment]] requires a new primary-source audit.

Primary-source map of products consumers confuse with Pytxo or choose instead. Pytxo is a **local agent hypervisor / control plane**: schedules headless coding agents (Claude Code, Codex, Cursor CLI, …) in PTYs with Signal / Blast / Race shields, optional Desktop (structural telemetry, not terminal walls), Flow mission planning, BYOK. It is **not** a multi-terminal browser IDE and **not** a single-vendor agent.

Companion category map: [[multi-agent-orchestration-landscape]]. Deep compares: [[pytxo-vs-claude-agent-teams]], [[pytxo-vs-github-copilot-app]], [[pytxo-vs-ade-virtual-workspace]].

**Research date:** 2026-07-26. Pricing and plan names churn monthly — re-check cited URLs before shipping GTM copy.

## Category frame (how to read overlaps)

| Bucket | What buyers think they bought | Fatal confusion with Pytxo? |
|--------|-------------------------------|-----------------------------|
| **Vendor coding agent** (Claude Code, Codex, Cursor Agent) | One agent that writes code | High — “I already have an agent” |
| **Vendor agent desktop** (Copilot app) | Control center for *their* agents | High — closest UX peer |
| **Cloud VM / autonomous engineer** (Devin, Cursor Cloud Agents, Amp orbs) | Agents that keep working when laptop closed | Medium — different locus of execution |
| **ADE / workroom** (BridgeSpace, Emdash) | Multi-pane terminals + kanban | High visual overlap; a different supervision model |
| **Cloud control plane** (Warp Oz) | Fleet ops for cloud agents | Medium — orchestration word collision |
| **IDE assistant / OSS agent** (Cline, Aider, Cody, Tabnine) | Chat / edit / BYOK in editor | Low as category peer; high as “why another tool?” |
| **Multi-agent SDK** (CrewAI, AutoGen → MAF) | Build your own agent app | Low — builders vs operators |
| **Governance “hypervisor”** (Microsoft AGT) | Policy rings for enterprise agents | Lexical only |

Pytxo’s durable wedge (without claiming unique sandboxing): **heterogeneous local CLI orchestration + Race collision policy + Signal structural context + Blast approve-to-flush + BYOK**, with Desktop focused on summaries, review, and decisions.

---

## Vendor coding agents (execution yard peers — Pytxo *runs* these)

### Claude Code / Claude Agent Teams (Anthropic)

| | |
|--|--|
| **What** | Anthropic’s coding agent across terminal, IDE, Slack, web; Agent Teams coordinate multiple Claude Code instances (lead + teammates + shared tasks). |
| **Who** | Individual Claude subscribers and Team/Enterprise orgs living in Anthropic’s stack. |
| **Pricing (public)** | Bundled into Claude plans — Pro ~$17–20/mo, Max 5x $100, Max 20x $200; Team Premium seats include Claude Code; Enterprise seat + usage ([claude.com/product/claude-code](https://claude.com/product/claude-code), [claude.com/pricing](https://claude.com/pricing)). |
| **Fatal overlap** | Parallel agents on one repo; “teams”; Agent view / routines / computer use expand supervision surface. |
| **Practical difference** | Pytxo coordinates Claude, Codex, and other CLIs under one path-ownership policy. Anthropic notes that Agent Teams teammates **do not** get automatic worktree isolation and can overwrite the same files; its docs also make the added token use explicit ([code.claude.com/docs/en/agent-teams](https://code.claude.com/docs/en/agent-teams)). |
| **Honest Pytxo weakness** | Claude alone is enough for many Claude-only shops; Agent Teams are free with the seat (experimental flag); Anthropic ships Agent view and workflows faster than a small control plane. |

Agent Teams: experimental (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`), significantly more tokens than one session, in-process or split panes — not a multi-CLI hypervisor.

### Cursor (Agent / Background Cloud Agents / Bugbot)

| | |
|--|--|
| **What** | AI-native IDE (fork of VS Code) with Agent, cloud agents, MCP/skills/hooks, and Bugbot PR review. |
| **Who** | Indie and team developers who want editor + agent as one product. |
| **Pricing (public)** | Hobby free; Individual from $20/mo (Pro / Pro+ / Ultra); Teams $40/user/mo; Enterprise custom. Cloud agents + Bugbot on usage-based billing ([cursor.com/pricing](https://cursor.com/pricing), [cursor.com/docs/bugbot](https://cursor.com/docs/bugbot)). |
| **Fatal overlap** | “Background agents,” parallel work, MCP hub language, spend limits — buyers say “I already run agents in Cursor.” |
| **Practical difference** | Pytxo coordinates **agents outside Cursor’s product** (Claude Code CLI, Codex, …) under one Race/Blast policy and can run locally with BYOK. Cursor combines its agent and editing experience in one product. |
| **Honest Pytxo weakness** | Cursor owns the daily editing surface and distribution; Cloud Agents + Bugbot close the “async agent” loop inside one bill; Continue’s acqui-hire consolidates OSS BYOK refugees into Cursor’s orbit ([continue.dev](https://continue.dev/)). |

### GitHub Copilot (coding agent + Copilot app)

| | |
|--|--|
| **What** | GitHub’s pair-programmer / agent stack; **Copilot app** is an agent-native desktop control center (My Work, worktree-per-session, canvases, Agent Merge, local + cloud sandboxes). |
| **Who** | Developers and orgs already on GitHub + Copilot seats. |
| **Pricing (public)** | Seat + **GitHub AI Credits** usage (from June 2026): Pro ~$10/mo, Pro+ ~$39, Business ~$19/user, Enterprise ~$39/user; completions unlimited on paid; agent/chat consume credits ([github.com/features/copilot](https://github.com/features/copilot), [github.blog usage-based billing](https://github.blog/news-insights/company-news/github-copilot-is-moving-to-usage-based-billing/)). Copilot app: technical preview on Pro/Pro+/Business/Enterprise ([GitHub blog Build 2026](https://github.blog/news-insights/product-news/github-copilot-app-the-agent-native-desktop-experience/)). |
| **Fatal overlap** | Closest **category peer** to Pytxo Desktop: parallel sessions, worktrees, sandboxes, approvals, merge automation. |
| **Practical difference** | Pytxo coordinates heterogeneous CLIs and adds Race ownership, Signal skeletons, and Galaxy HITL outside a GitHub-only workflow. Copilot is integrated directly with GitHub identity, pull requests, and billing. |
| **Honest Pytxo weakness** | Copilot ships worktree-per-session and Agent Merge as a productized GitHub workflow; do **not** claim Pytxo invented worktree isolation ([[pytxo-vs-github-copilot-app]]). Its distribution and CI integration are a strong fit for GitHub-native teams. |

### OpenAI Codex / ChatGPT coding agent

| | |
|--|--|
| **What** | OpenAI coding agent: CLI, IDE extension, macOS/Windows app, cloud tasks, code review; OS-level sandbox + approval policies. |
| **Who** | ChatGPT subscribers and API-key users wanting OpenAI’s coding harness. |
| **Pricing (public)** | Included in ChatGPT Free/Go/Plus/Pro/Business/Enterprise; Plus ~$20; Pro from ~$100 with 5×/20× usage; API key = token rates, no cloud features ([chatgpt.com/codex/pricing](https://chatgpt.com/codex/pricing/), [developers.openai.com/codex/pricing](https://developers.openai.com/codex/pricing)). |
| **Fatal overlap** | Local sandbox + approval mental model mirrors Blast/HITL language; CLI is an execution-yard peer. |
| **Practical difference** | Pytxo can run Codex alongside other CLIs under shared ownership and review policy. Codex provides the coding agent and its own sandbox and approval model. |
| **Honest Pytxo weakness** | Codex’s Seatbelt/bwrap/Windows sandbox story is stronger OS containment than Pytxo’s current worktree/copy-layer Blast (kernel ProjFS/FUSE still north star). |

### Devin (Cognition)

| | |
|--|--|
| **What** | Autonomous software engineer product: Devin Desktop, Devin Cloud, DeepWiki, Review, Slack/Linear integrations — Cognition’s agent, not your mixed CLIs. |
| **Who** | Teams buying a managed “AI engineer” seat/quota. |
| **Pricing (public)** | Free / Pro $20 / Max $200 / Teams $80 min (+ $40 full seats) / Enterprise custom; daily+weekly quotas, overage at API pricing ([devin.ai/pricing](https://devin.ai/pricing), [docs.devin.ai/admin/billing](https://docs.devin.ai/admin/billing)). |
| **Fatal overlap** | “Agents ship PRs while I sleep”; concurrent sessions; cloud agents. |
| **Practical difference** | Pytxo lets teams keep their chosen Claude, Codex, and other CLIs with local BYOK execution. Devin offers a managed agent and cloud workflow as one product. |
| **Honest Pytxo weakness** | Devin owns end-to-end autonomous loop + cloud compute brand; Pytxo requires users to already run (or install) yard agents. |

---

## IDE / terminal agents (adjacent — usually not hypervisors)

### Aider

| | |
|--|--|
| **What** | Open-source terminal pair-programmer: codebase map, git auto-commits, multi-LLM BYOK ([aider.chat](https://aider.chat/)). |
| **Who** | CLI-native developers who want transparent git diffs and provider choice. |
| **Pricing** | Free OSS; cost = provider tokens. |
| **Practical comparison** | Both support local BYOK coding. Pytxo adds multi-agent Race/Blast/Flow and Desktop supervision; Aider is lighter to adopt for single-agent git loops. |

### Continue (acquired by Cursor)

| | |
|--|--|
| **What** | Was the leading open-source IDE coding agent / BYOK extension. |
| **Status** | **Acquired by Cursor**; product winding down; OSS codebase remains ([continue.dev](https://continue.dev/)). |
| **Implication for Pytxo** | BYOK OSS IDE niche consolidating; refugees may try Cline/Aider **or** need a **control plane** that isn’t another IDE. Positioning: “your agents still run; Pytxo coordinates them.” |

### Cline

| | |
|--|--|
| **What** | Open-source coding agent runtime: IDE extension, CLI, SDK; Plan/Act; MCP; multi-agent teams/schedules; BYOK ([cline.bot](https://cline.bot/)). |
| **Who** | Developers wanting Apache-2.0 agent without vendor IDE lock-in. |
| **Pricing** | Free OSS; pay model providers; commercial surfaces via Cline ecosystem. |
| **Practical comparison** | Both cover Plan/Act, multi-agent work, and MCP. Pytxo coordinates Cline, Claude Code, and Codex under shared Race policy; Cline offers one agent runtime across its supported surfaces. |

### Roo Code

| | |
|--|--|
| **What** | VS Code agent extension marketed as “a whole dev team of AI agents in your editor” ([github.com/RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code)). |
| **Who** | VS Code users wanting modes / multi-agent-in-editor workflows. |
| **Pricing** | Extension historically free/OSS; cloud add-ons when offered — verify live site before quoting. |
| **Practical comparison** | Roo Code keeps multi-agent work in VS Code. Pytxo runs headless PTYs with a separate structural Desktop, which adds another install. |

### Windsurf (Codeium)

| | |
|--|--|
| **What** | AI IDE with Cascade agent, local/cloud agents, tab complete. |
| **Who** | Developers wanting Cursor-class IDE alternative. |
| **Pricing (reported / verify)** | Free + Pro ~$20 / Max ~$200 / Teams ~$40/user with daily/weekly quotas (third-party summaries cite windsurf pricing — confirm on [windsurf.com](https://www.windsurf.com/) before GTM). |
| **Practical comparison** | Windsurf integrates agents into the editor. Pytxo coordinates mixed CLIs outside any one IDE. |

### Amp (Sourcegraph Amp / ampcode)

| | |
|--|--|
| **What** | “Frontier agent” — web, terminal, phone; **orbs** = remote machines that keep working; plugins; subscription or pay-as-you-go ([ampcode.com](https://ampcode.com/)). |
| **Who** | Power users optimizing for unsupervised remote agent runs. |
| **Pricing (public)** | Megawatt $20/mo (orbs + $20 agent usage); Gigawatt $200/mo; PAYG still available ([ampcode.com/news/subscriptions](https://ampcode.com/news/subscriptions)). |
| **Practical comparison** | Amp emphasizes remote, unsupervised agents and orbs. Pytxo emphasizes local silicon and mixed harnesses; Amp, Oz, or Devin are better fits when laptop-closed scale is the priority. |

### Sourcegraph Cody

| | |
|--|--|
| **What** | Enterprise AI coding assistant grounded in Sourcegraph code graph / search ([sourcegraph.com/cody](https://sourcegraph.com/cody)). |
| **Who** | Large-codebase enterprise on Sourcegraph. |
| **Pricing** | Enterprise / Sourcegraph-bundled (contact sales; individual Free/Pro discontinued in 2025 per market reports — confirm with Sourcegraph). |
| **Practical comparison** | Both address large-repository context. Pytxo Signal provides local AST skeletons for agent context; Cody provides deeper Sourcegraph-backed monorepo search today. |

### Tabnine

| | |
|--|--|
| **What** | Privacy-first code assistant + agentic platform; on-prem / private LLM emphasis ([tabnine.com/pricing](https://www.tabnine.com/pricing/)). |
| **Who** | Regulated enterprises needing air-gap / governance. |
| **Pricing** | Quote-based Code Assistant and Agentic Platform (per-user/month annual); BYO LLM unlimited; Tabnine-hosted LLM = provider + handling fee. |
| **Practical comparison** | Both use agent-governance language. Pytxo coordinates mixed CLIs; Tabnine is designed for regulated, private, and on-premise deployments that Pytxo does not yet cover. |

---

## Multi-agent frameworks (builders, not coding hypervisors)

### CrewAI

| | |
|--|--|
| **What** | Role-based multi-agent framework (“crew” of specialists) for building agent apps. |
| **Who** | Builders prototyping business/agent workflows. |
| **Pricing** | OSS + CrewAI enterprise/cloud offerings (check crewai.com). |
| **Overlap** | Marketing uses “teams of agents” — consumers may conflate with coding swarms. |
| **Practical difference** | Pytxo operators run **existing** coding CLIs; CrewAI users build new agent applications. |
| **Weakness** | If buyer wants to *author* agent graphs in Python, CrewAI/LangGraph fit; Pytxo does not. |

### AutoGen → Microsoft Agent Framework

| | |
|--|--|
| **What** | Research multi-agent conversations (AutoGen) consolidating into **Microsoft Agent Framework** for production Azure/.NET/Python stacks. |
| **Who** | Enterprise builders on Microsoft AI stack. |
| **Overlap** | “Orchestration,” group chat, Magentic patterns. |
| **Practical difference** | Pytxo is a local coding-CLI hypervisor; Microsoft Agent Framework is an SDK for building agent applications. |
| **Weakness** | Procurement-aligned Microsoft stack; Pytxo is not an Azure agent SDK. |

---

## ADE / multi-terminal workrooms

### BridgeSpace (BridgeMind)

| | |
|--|--|
| **What** | Agentic Development Environment: up to **16 GPU-accelerated agent terminals**, Kanban dispatch, BridgeSwarm (coordinator/builders/scouts/reviewers), BridgeMCP, editor+browser in one Tauri desktop ([bridgemind.ai/products/bridgespace](https://www.bridgemind.ai/products/bridgespace)). |
| **Who** | “Vibe coding” users who want a visual swarm room. |
| **Pricing** | Included with paid BridgeMind plans (Basic+) — compare on BridgeMind pricing. |
| **Fatal overlap** | Multi-agent + Claude Code/Codex/Cursor in terminals; BridgeSwarm claims **file ownership** so agents don’t collide — Race Shield adjacency. |
| **Practical difference** | BridgeSpace emphasizes direct observation through a multi-terminal workroom. Pytxo keeps agents headless and uses Desktop for ownership, review, approvals, and cost ([[beyond-the-ade]], [[pytxo-vs-ade-virtual-workspace]]). |
| **Honest Pytxo weakness** | BridgeSpace demos better; Swarm + Kanban feel like “mission control” immediately. Pytxo must win on *after* the demo: collisions, cost, mixed CLI policy. |

### Emdash (YC W26)

| | |
|--|--|
| **What** | Open-source ADE: parallel agents in **git worktrees**, 25–34+ CLI agents, scheduling, in-app browser, issue integrations, remote SSH ([emdash.ai](https://emdash.ai/), [emdash.ai/docs](https://emdash.ai/docs)). |
| **Who** | Developers wanting OSS multi-agent workroom without BridgeMind lock-in. |
| **Pricing** | Free / open-source (verify commercial add-ons). |
| **Fatal overlap** | Explicitly multi-CLI + worktrees + review — very close to “local orchestration UI.” |
| **Practical difference** | Emdash centers a worktree-based multi-agent workroom. Pytxo centers a Rust scheduler, Race registry, Signal context, prepared Apply, and an MCP-facing hypervisor. |
| **Honest Pytxo weakness** | Emdash already speaks “any coding agent + worktrees”; Pytxo must show **policy + telemetry moats**, not “we also run Claude Code.” |

---

## Cloud control planes & “hypervisor” lexical peers

### Warp Oz

| | |
|--|--|
| **What** | Cloud orchestration platform for coding agents: triggers → tasks → environments/hosts; CLI/API/SDK/web; multi-agent parent/child; harnesses include Warp Agent, Claude Code, Codex ([docs.warp.dev/platform/overview](https://docs.warp.dev/platform/overview/), [warp.dev/blog/oz](https://www.warp.dev/blog/oz-orchestration-platform-cloud-agents)). |
| **Who** | Teams scaling **cloud** agent fleets with Slack/GitHub/CI triggers. |
| **Pricing** | Warp credits / team plans (see Warp billing). |
| **Fatal overlap** | “Orchestration platform,” heterogeneous harnesses, observability — strongest **cloud** peer to Pytxo’s control-plane story. |
| **Practical difference** | Pytxo’s ordinary path is local-first and does not require a cloud control plane. Oz supports laptop-closed, event-triggered fleet operations and hosted team audit trails. |
| **Honest Pytxo weakness** | Oz is the stronger fit when hosted triggers, remote execution, and shared fleet operations are primary requirements. |

### Microsoft Agent Governance Toolkit — “Agent Hypervisor”

| | |
|--|--|
| **What** | Enterprise governance SDK: execution rings 0–3, kill switch, saga rollback — **policy hypervisor**, not a coding-agent runner ([microsoft.github.io/agent-governance-toolkit](https://microsoft.github.io/agent-governance-toolkit/packages/agent-hypervisor/)). |
| **Who** | Platform/SRE teams governing autonomous agents in enterprise systems. |
| **Overlap** | The phrase **agent hypervisor** — SEO/lexical collision only. |
| **Pytxo stance** | Own the phrase for **local coding-CLI scheduling**; clarify “not Microsoft AGT” in docs FAQ if needed. Adjacent inspiration for [[permission-profile-engine]] rings (DeepSpace→Supernova), not a competitor for consumers. |

---

## Matrix — confusion risk vs Pytxo advantage

| Product | Confusion | Pytxo fit | Prefer the competitor when… |
|---------|-----------|-------------------------------|-------------------|
| Claude Agent Teams | High | Mixed CLI + Race | Claude-only is enough |
| Cursor Agent/Cloud | High | Outside-Cursor yard + BYOK policy | User lives only in Cursor |
| Copilot app | Very high | Heterogeneous + Signal/Race | Org is GitHub+Copilot native |
| Codex | Medium | Runs Codex in mixed swarm | OpenAI-only + OS sandbox enough |
| Devin | Medium | Keep your agents | Want managed AI engineer |
| Amp orbs | Medium | Local-first mixed | Want remote unsupervised |
| BridgeSpace | High (anti) | Throughput, not panes | Want 16-terminal vibe room |
| Emdash | High | Policy moats vs ADE shell | Want OSS ADE worktrees UI |
| Warp Oz | Medium | Local hypervisor | Cloud fleet is the job |
| Cline/Aider | Low–med | Swarm control plane | Single-agent edits |
| CrewAI/MAF | Low | Operator plane | Builder SDK needed |
| Cody/Tabnine | Low | Not enterprise search/airgap | Procurement mandates them |

---

## Recommendations for pytxo.com/docs consumers

### (1) The 4–6 comparisons that matter most

Ship / keep these as first-class compare pages (already have 1–3):

1. **Pytxo vs Claude Agent Teams** — same-vendor swarm vs cross-CLI hypervisor; cite Anthropic’s no-worktree warning.  
2. **Pytxo vs GitHub Copilot app** — closest supervision peer; honesty on worktrees.  
3. **Pytxo vs ADE virtual workspaces** (BridgeSpace / Emdash archetype) — direct terminal observation vs structural control.
4. **Pytxo vs Cursor Cloud Agents** — cloud VM agents inside Cursor vs local mixed-CLI yard (new page recommended).  
5. **Pytxo vs Warp Oz** — cloud fleet control plane vs local-first hypervisor (new page recommended).  
6. **Pytxo vs Devin / Amp (autonomous remote)** — managed/remote engineer vs coordinate-what-you-already-run (optional sixth).

Deprioritize for docs nav: CrewAI/AutoGen (wrong buyer), Tabnine/Cody (procurement niche), Aider (mention in FAQ only).

### (2) Positioning language — “dangerous / inevitable” without lying

Use:

- **“Runs the agents you already pay for.”** (Claude Code, Codex, Cursor CLI — not a seventh chat UI.)
- **“Local agent hypervisor.”** — schedule, isolate, approve, observe. Not “the only multi-agent.”
- **“Collisions are a systems problem.”** — Race Shield; Claude’s own docs admit overwrite risk.
- **“Approve to flush.”** — Blast as default trust posture; don’t claim unique sandbox vs Copilot/Codex.
- **“Structural telemetry, not terminal walls.”** — Desktop Focus/Ops vs ADE grids.
- **“BYOK. Your keys. Your silicon.”** — against credit-metered cloud yards when local is enough.

Avoid:

- “Only sandbox / only multi-agent / invented worktrees.”
- Fake 10× stats; cinematic ADE hero copy on Desktop ([[market-ready-polish-research]]).
- Claiming Microsoft AGT equivalence.

Inevitable frame (honest): **Stacks are already mixed.** Cursor + Claude Code + Codex is common. Vendor teams optimize *their* agent. Pytxo is the layer that appears when parallelism becomes a **collision and cost** problem — same way hypervisors appear when you run more than one workload.

### (3) Product UX differentiators that create daily-driver habit

Not feature lists — **impulsive reopen loops**:

1. **10-second status truth** — What’s running / what needs me / what’s sandboxed / what’s it costing? (table stakes from Copilot Sessions + Cursor spend — [[market-ready-polish-research]]).
2. **Approval as the dopamine beat** — one clear Blast flush / Galaxy HITL queue; users return to *decide*, not to watch scrollback.
3. **Race claim visibility** — show path locks and blocked agents; make collisions *felt* then *solved* (BridgeSwarm’s ownership claim is the competitive pressure).
4. **Flow mission → yard dispatch** — plan once, agents fan out; return for merge/approve, not for babysitting 16 panes.
5. **Structural Focus graph** — Signal nodes/edges as the home screen (Desktop 2), so reopening feels like *seeing the system*, not another chat.
6. **Cost burn strip** — live token/spend per agent; Agent Teams’ token warning becomes a habit hook if Pytxo makes spend scannable.

---

## Sources (primary preferred)

| Topic | URL |
|-------|-----|
| Claude Code product / pricing | https://claude.com/product/claude-code · https://claude.com/pricing |
| Claude Agent Teams docs | https://code.claude.com/docs/en/agent-teams |
| Cursor pricing / Bugbot | https://cursor.com/pricing · https://cursor.com/docs/bugbot |
| Copilot app announcement | https://github.blog/news-insights/product-news/github-copilot-app-the-agent-native-desktop-experience/ |
| Copilot usage billing | https://github.blog/news-insights/company-news/github-copilot-is-moving-to-usage-based-billing/ |
| Codex pricing | https://chatgpt.com/codex/pricing/ · https://developers.openai.com/codex/pricing |
| Devin pricing / billing | https://devin.ai/pricing · https://docs.devin.ai/admin/billing |
| Aider | https://aider.chat/ |
| Continue acquisition notice | https://continue.dev/ |
| Cline | https://cline.bot/ |
| Amp + subscriptions | https://ampcode.com/ · https://ampcode.com/news/subscriptions |
| Sourcegraph Cody | https://sourcegraph.com/cody |
| Tabnine pricing | https://www.tabnine.com/pricing/ |
| BridgeSpace / BridgeSwarm | https://www.bridgemind.ai/products/bridgespace · https://www.bridgemind.ai/bridgeswarm |
| Emdash | https://emdash.ai/ · https://emdash.ai/docs |
| Warp Oz | https://docs.warp.dev/platform/overview/ · https://www.warp.dev/blog/oz-orchestration-platform-cloud-agents |
| Microsoft Agent Hypervisor | https://microsoft.github.io/agent-governance-toolkit/packages/agent-hypervisor/ |
| Pytxo internal peers | [[multi-agent-orchestration-landscape]], [[product-vision]], [[beyond-the-ade]] |

---

## Revisit triggers

- Copilot app GA / pricing changes
- Claude Agent Teams leaving experimental
- Warp Oz harness list / local execution posture
- Emdash / BridgeSpace file-ownership & isolation claims
- Amp orb vs local narrative winning mindshare
- Any new product literally named “agent hypervisor” for coding CLIs

Back: [[MOC-home]] · [[pytxo-improvement-research]]
