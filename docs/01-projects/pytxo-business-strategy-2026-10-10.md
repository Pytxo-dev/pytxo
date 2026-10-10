---
title: Pytxo business strategy October 2026
slug: pytxo-business-strategy-2026-10-10
status: draft
tags: [project, strategy, gtm, pricing, competitive, research]
audience: [human, agent]
layer: meta
created: 2026-10-10
updated: 2026-10-10
related: [[product-vision]], [[positioning]], [[pytxo-commit-layer-alignment]], [[competitive-landscape-2026-07]], [[beta-competitor-research-2026-09-05]], [[tiers-hobbyist-pro-max]], [[gtm-open-source-loop]], [[dodo-mor-integration]], [[PYTXO_OPEN_QUESTIONS]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0036-effect-contract-commit-boundary]]
---

# Pytxo business strategy — October 2026

> [!NOTE]
> Dated research and recommendation note, longer than an atomic note by
> design. It is input for Matt's decision, not an approved decision. Nothing
> here authorizes a release, price, publication, redesign, or external action.
> External figures were checked 2026-10-10; most come from press and vendor
> pages and are labelled where they are secondary or unverified.

## 0. The answer in one screen

1. **The layer Pytxo currently sells is free and crowded.** "Run several
   coding agents in parallel" is now a commodity: Orca (MIT, free,
   macOS/Windows/Linux, 27 agents, ~89k GitHub stars), Conductor (free local
   tier, $22M raised), Superset, T3 Code (MIT, ~27k stars), Emdash, Composio's
   Agent Orchestrator, Claude Squad, plus every vendor's own app (Codex app on
   Windows since March, Claude Code Projects since September, Cursor 2.0,
   GitHub Agent HQ, VS Code, Zed). Vibe Kanban had ~28k stars and thousands of
   daily users and shut down in April 2026 because "the vast majority are free
   users and we couldn't find a business model." A better orchestrator desktop
   app is not a business.
2. **The money and the pain are at the end of the run.** Review time per PR is
   up 91%, AI PRs land at 32.7% vs 84.4% for human PRs, 61% of agent PRs get no
   review, feature-branch throughput is up 59% while main-branch throughput is
   flat. CodeRabbit reached a $1.5B valuation on review comments alone;
   "control over agents" startups raised $435M in five months. Buyers already
   pay $20–30 per developer per month for review tooling.
3. **Pytxo's real, tested assets are exactly end-of-run primitives:**
   file-ownership scheduling, checks on the combined change, exact-byte Apply
   with stale refusal, journaled recovery, enforcement receipts, a run ledger.
   The commit-layer thesis in [[product-vision]] is right. The packaging (a
   Windows desktop orchestrator sold to individuals) is wrong.
4. **Recommendation: re-aim the company at "the landing layer for
   agent-written changes."** Pytxo decides what agent work may become real,
   proves it, and can undo it, for output from *any* agent or workspace (Orca,
   Conductor, Codex cloud, Copilot, Devin, Claude Code Projects). Ship it as a
   CLI gate, a GitHub App/Action, an MCP tool, and the existing Desktop as the
   local cockpit. Free for individuals and public repos; per-developer pricing
   for private team repos; evidence and compliance for enterprises.
5. **Three horizons.** H1 (now–March 2027): launch, 100 real users, first
   paying teams, shadow-mode gate. H2 (2027): the evidence ledger and
   compliance exports. H3 (2028+): effect contracts beyond the repository, the
   north star in [[pytxo-commit-layer-alignment]].
6. **Hard gates.** If fewer than 5 of 20 contacted developers run Pytxo twice
   within two weeks of launch, change the message before building anything
   else. If no team asks about paying within 90 days of the gate shipping,
   narrow or stop.
7. **Biggest risks.** GitHub absorbing the gate into Agent HQ; Anthropic's
   subscription terms tightening around parallel headless use; zero audience;
   one under-18 operator with no sales motion.

## 1. Where Pytxo is today (evidence, not opinion)

| Dimension | Observed 2026-10-10 |
|---|---|
| Product | v1.2.5 public beta, MIT since 7 Oct. 313 commits, 1,681 tracked files, ~137k lines of Rust, ~31k lines of Svelte/TS, 1,051 Rust test functions, 14 crates, 3 services (Link, proxy, cloud sandbox), Windows x64 MSI, CLI on three platforms |
| Proof | Live run on the released v1.2.4 exe: Claude Code split 7 tasks; Claude Code + Codex + Cursor Agent 7/7; 14 files reviewed = applied; stale refusal; 22/22 fixture tests (`LAUNCH.md`, gate 3). Antigravity and OpenCode not working end to end |
| Adoption | GitHub: 0 stars, 0 forks, 1 page view in 14 days, 893 clones from 64 uniques (consistent with CI). Release assets: a few hundred downloads across 25 releases, most consistent with CI and acceptance runs; the 9 Oct session estimated ~13 real downloads. npm: 570 downloads 1 Sep–10 Oct, 362 of them on 7–8 Oct (release and acceptance days) |
| Audience | X 0 followers; HN 4 karma with one flagged comment; Reddit posts auto-removed for low karma; Discord size unknown; nothing posted for the launch yet |
| Billing | Link service exists with Paddle and Dodo webhook adapters, entitlements, migrations. No live checkout, no approved Dodo brand, no public prices or legal pages. Operator is under 18, so the legal seller is the MBCZ guardian structure ([[dodo-mor-integration]]) |
| Launch kit | Fact sheet with do-not-claim list, Product Hunt listing and gallery, film v8, X drafts, HN/Reddit research. Matt writes HN and Reddit wording himself |
| Prior verdict (9 Oct) | "Don't kill it, change how you sell it, stop building until people use it." Still correct. This note adds: change *what* you sell |

Six months of work produced a real, tested control layer and zero users. The
risk is not the code. It is that the thing being sold has no buyer and the
thing that has a buyer is not yet packaged.

## 2. Market map, October 2026

### 2.1 Parallel-agent workspaces (Pytxo's current shelf)

| Product | Model | Platforms | Price | Signal |
|---|---|---|---|---|
| [Orca](https://www.onorca.dev/) | OSS, MIT | macOS, Windows, Linux, mobile | Free | [~89k stars](https://github.com/stablyai/orca); 27 agents; "fan one prompt across 5 agents, compare, merge the winner" |
| [Conductor](https://www.conductor.build/pricing) | Closed | macOS; cloud workspaces on Vercel sandboxes | Free local; Pro $50/mo; Teams $60/user; Enterprise | [$22M Series A](https://tracxn.com/d/companies/conductor/__GDvVaFT0iP5pizAQSml7d0e-1TcgZIeanC09x60Mwto) (date unverified); no Windows app |
| [Superset](https://superset.sh/pricing) | Source-available (ELv2) | macOS first | Free; Pro $20/user; Enterprise | ~15k stars; automations, SDK, MCP, remote |
| [T3 Code](https://github.com/pingdotgg/t3code) | OSS, MIT | macOS, Windows, Linux | Free | ~27k stars |
| [Emdash](https://github.com/generalaction/emdash) | OSS, Apache-2.0 | Desktop | Free | ~6k stars; YC W26; 30+ agents |
| [Composio Agent Orchestrator](https://github.com/ComposioHQ/agent-orchestrator) | OSS, MIT | CLI/tmux/Docker | Free | ~7k stars; autonomous PR lifecycle (CI fail → agent) |
| Claude Squad | OSS, AGPL | Terminal | Free | ~8.6k stars |
| [Vibe Kanban](https://vibekanban.com/shutdown) | OSS, Apache-2.0 | Local + cloud | Was freemium | ~28k stars; company shut down April 2026, "vast majority are free users" |
| 1code (21st.dev) | Startup | — | — | [YC W26](https://www.ycombinator.com/companies/1code-21stdev/jobs), "orchestration layer for coding agents" |

Takeaways: the free, cross-platform, open-source option already has two
orders of magnitude more adoption than Pytxo; the funded closed option is
Mac-only; the one company that tried to charge for this layer closed. Windows
is a real gap (Conductor, Superset, Codex app initially), but Orca and T3 Code
already fill it for free.

### 2.2 Vendor-bundled orchestration

- **OpenAI Codex app**: parallel agents in worktrees, [macOS Feb 2026](https://openai.com/index/introducing-the-codex-app/), Windows March 2026, now inside the ChatGPT desktop app with computer use.
- **Anthropic Claude Code Projects**: [relaunched 17 Sep 2026](https://aichief.com/news/claude-code-relaunches-projects-to-manage-multiple-ai-agents-in-the-cloud/) as parallel cloud threads with a coordinator; each thread is a branch and overlaps surface as ordinary merge conflicts; local support "coming soon" (secondary coverage of The Verge).
- **GitHub Agent HQ**: [Claude and Codex in public preview](https://github.blog/news-insights/company-news/pick-your-agent-use-claude-and-codex-on-agent-hq/) on Copilot Pro+/Enterprise since 4 Feb 2026; mission control, per-agent identities, branch controls.
- **Cursor 2.0** parallel agents and cloud agents; **VS Code** [multi-agent](https://code.visualstudio.com/blogs/2026/02/05/multi-agent-development); **Zed** parallel agents (manual merge).

Every vendor's orchestration is bundled "free" into a subscription the
developer already pays. None of them is neutral, and each produces N branches
that a human still has to land.

### 2.3 Autonomous engineers and agent platforms

Cognition/Devin (~$492M ARR estimate, $26–48B valuation reports), Factory
([$200M at $5B](https://www.metirai.com/blog/factory-ai-200-million-5-billion-coding-agents-2026), Pro $20 / Plus $100 / Max $200), Amp, Jules, Warp Oz
(Build ~$18–20, Max ~$180–200, Business ~$45–50/seat, secondary sources).
Cursor reports [~$4B annualized revenue](https://dealroom.co/news/134107-cursor-tops-4b-annualized-revenue/) with ~75% enterprise. Claude Code
passed a reported $2.5B run-rate in February 2026 (press, not primary). The
agents themselves are a capital-intensive, vendor-owned market. Pytxo should
never compete here.

### 2.4 The review and landing layer (where Pytxo belongs)

| Product | What it sells | Price | Signal |
|---|---|---|---|
| [CodeRabbit](https://sacra.com/c/coderabbit/) | AI review comments | Pro $24–30/dev/mo; free for public repos | [$143M Series C at $1.5B](https://ecvc.wsgr.com/news/wilson-sonsini-advises-coderabbit-on-dollar143-million-series-c-funding), Aug 2026; ~$40M ARR estimate; 2M reviews/week |
| Graphite | Stacked PRs, merge queue, AI review | — | [Acquired by Cursor](https://research.contrary.com/company/graphite), Dec 2025 |
| [Mergify](https://mergify.com/pricing) | Merge queue, CI insights | ~$21/seat/mo; free ≤5 users | Positions against GitHub's FIFO single queue |
| GitHub merge queue | Speculative batch testing | Bundled | FIFO, one queue, no agent provenance, no cross-PR ownership |
| Greptile, Qodo | AI review | per seat | Review, not landing |

Nobody in this row verifies the *combined* result of several agents' work
with provenance, detects semantic ownership collisions across concurrent
agent PRs, or lands exact bytes with a stale-safe, journaled, revertible
receipt. That is what Pytxo already does locally.

### 2.5 Agent governance and control planes (the money)

Zenity $125M Series C, Onyx $113M Series B, Alice $140M, Island $400M at
$6.4B ("agentic control plane"), [Rein $25M](https://www.calcalistech.com/ctechnews/article/rkf52cnimg), [Geordie $30M](https://fortune.com/2026/05/28/geordie-security-governance-ai-agents/), AIR $50M seed; [$435M across 12 deals](https://cvj.ai/briefing/finance-news/ai-agent-security-startups-raise-435m-as-governance-becomes-the-bottleneck/) April–September 2026; "eleven of thirty-three Q3 rounds sell control over agents rather than agents" ([tracker](https://gravity.fast/blog/ai-agent-funding-tracker-q3-2026/)). Microsoft [Entra Agent ID](https://learn.microsoft.com/en-za/entra/agent-id/whats-new-agent-id) and Okta for AI Agents make agent identity a procurement line.

These are enterprise-security companies with sales teams. Pytxo cannot
out-sell them head-on. It can reach the same buyer from below, through the
developer workflow, with an artifact they need (a receipt for every landed
agent change).

### 2.6 Demand signals

- Review bottleneck: [CircleCI 2026](https://blog.codacy.com/ai-breaking-code-review-how-engineering-teams-survive-pr-bottleneck): feature-branch throughput +59% YoY, median main-branch throughput down; Faros AI: PR review time +91%, PR size +154%; [LinearB 2026](https://www.kunalganglani.com/blog/ai-coding-assistant-team-adoption): AI PRs land at 32.7% vs 84.4%; a 2026 study: [61% of agent PRs receive no review](https://leaddev.com/ai/as-ai-helps-us-write-more-code-whos-catching-the-bugs). All secondary summaries; verify against the primary reports before public use.
- Trust: [Stack Overflow 2026](https://itbrief.co.uk/story/stack-overflow-survey-shows-developers-trust-ai-with-caveats): 66% use coding agents; 6.6% trust AI for important decisions; 78.7% say attribution matters.
- Incidents: [PocketOS production database and backups deleted in 9 seconds](https://oecd.ai/en/incidents/2026-04-27-6153) by a Cursor agent on Claude Opus 4.6 (April 2026) through a token found in an unrelated file; a Terraform production deletion attributed to Claude Code ([March 2026](https://oecd.ai/en/incidents/2026-03-07-79ec)); Replit (July 2025). Each is a missing gate between a proposed effect and a real one.
- Cost: Uber reportedly exhausted its 2026 AI coding budget in four months; Copilot's June 2026 usage billing produced 10–50x bill jumps for agentic users (secondary). Per-run budget enforcement, not alerts, is the emerging ask.
- Governance forecast: Gartner, [>40% of agentic AI projects canceled by 2027](https://www.outlookbusiness.com/deeptech/artificial-intelligence/over-40-of-agentic-ai-projects-will-be-scrapped-by-2027-says-gartner) for cost, value and risk controls.
- Regulation: EU AI Act high-risk logging duties [delayed to Dec 2027 / Aug 2028](https://datamatters.sidley.com/2026/06/22/eu-lawmakers-reach-provisional-agreement-to-delay-key-eu-ai-act-obligations/) by the Digital Omnibus. Not a 2026 forcing function, but a 2027 procurement question. SOC 2 change-management evidence (approval, review, tested rollback) is a today question.

### 2.7 Platform terms that matter

Anthropic's [Claude Code legal page](https://code.claude.com/docs/en/legal-and-compliance) (read 2026-10-10): subscription OAuth is "intended exclusively" for ordinary use of Claude Code and native Anthropic apps; third parties may not route requests through Pro/Max credentials or intermediate credentials; an end user may sign in to the *unmodified* Claude Code binary with their own subscription, including where a platform hosts it; "advertised usage limits for Pro and Max plans assume ordinary, individual usage"; Anthropic may enforce "without prior notice." OpenCode and others were cut off from subscription OAuth in 2026.

Pytxo is on the permitted side by construction: it runs the unmodified
vendor binary, under the user's own login, on the user's machine, with a clean
environment and no credential intermediation. Eight parallel headless
sessions on a Max plan is nonetheless exposed to a future "not ordinary
individual use" reading. Consequences: API-key BYOK must be first-class, a
per-run cost governor matters, and the business must not depend on any
vendor's auth policy. A gate that works on *outputs* (branches, PRs, patches)
is independent of how the agent ran.

## 3. Strategic read

**Where the value is.** Not parallelism (free), not the model (vendor), not
the editor (Cursor, VS Code, Zed), not the sandbox (Codex, E2B, Daytona). It
is the question in [[product-vision]]: which state transition may become real,
under whose authority, with what evidence, and with what recovery path. In
2026 that question is being asked loudly about code, by people who already
have budget, because agent PR volume has outrun review capacity.

**Why Pytxo can own it.** Three reasons, in order of durability:

1. Neutrality. Every vendor is building a walled garden (Anthropic's OAuth
   terms, the Codex app, Agent HQ behind Copilot seats). Teams run mixed
   stacks. The landing layer has to be Switzerland, and a Rust, MIT,
   local-first core from an independent is a credible Switzerland.
2. Depth already built. Ownership scheduling, combined checks, immutable
   review package, exact Apply, stale refusal, journaled recovery, enforcement
   receipts ([[ADR-0034-immutable-review-package-and-durable-apply]]). None of the
   workspace tools has this; none of the review tools has this.
3. Semantic edge. Signal Core's tree-sitter skeletons make *symbol-level*
   cross-PR collision detection possible. Merge queues see text; Pytxo can see
   that two agents changed the same function.

**Why the current packaging fails.** Individuals do not pay for coordination
(Vibe Kanban's lesson, Orca's gravity). "Hypervisor," "commit layer" and
"effect contract" do not sell to developers. Windows-only Desktop limits who
can try it. Six months with zero user contact is the actual defect.

**Why now.** 2026 is the year the review bottleneck became measurable and the
year money moved to agent control. The vendors are racing to add orchestration
but none has an incentive to build a neutral landing layer, and GitHub's
version is GitHub-only and Copilot-gated. The window is 12–18 months.

## 4. The plan: Pytxo lands agent work

### 4.1 One sentence

**Pytxo is the landing layer for agent-written changes: it decides what agent
work may become real, proves it, and can undo it, whatever agent or workspace
produced it.**

Public words: land, gate, checks, receipt, undo. Internal words stay internal
([[positioning]] language rules).

### 4.2 Product surfaces

| Surface | Status | Role |
|---|---|---|
| **Pytxo Desktop** (Windows MSI) | Shipping | Local cockpit: split, run, review, Apply. Keep. Stop treating it as *the* product. macOS only when users pull for it |
| **`pytxo gate`** (CLI + GitHub Action) | New | Takes N agent outputs (branches, PRs, patch sets) from any tool. Computes file and symbol ownership conflicts, builds the combined candidate, runs the project's checks on the combined state inside the customer's CI, emits a signed receipt, lands exact bytes or opens one combined PR, refuses on drift. **Shadow mode first**: observe and comment, never block |
| **Pytxo GitHub App** | New | A check run on every agent PR; cross-PR ownership collisions; combined-queue verification; provenance receipt in the PR; "Land via Pytxo." GitLab later |
| **Pytxo Ledger** (hosted) | New, the SaaS | Receipts, retention, search, SSO, policy, exports (Vanta/Drata, SIEM), in-toto-style attestations. Self-hosted option for enterprise |
| **MCP tool** (`pytxo-mcp`) | Exists | Agents call `land` themselves from Claude Code, Cursor, Codex. Distribution inside the agents, via the official MCP registry and vendor plugin directories |
| **Local run ledger + enforcement receipts** | Exists | The seed of Ledger; already distinguishes enforced, advisory, unavailable, bypassed |

Keep checks executing in the customer's CI. Pytxo orchestrates and attests;
GitHub Actions (or the customer's runner) computes. That keeps gross margin
near 90% and avoids the hosted-sandbox race.

### 4.3 What nobody else has (and what to protect)

1. **Combined-state verification with provenance.** "Each agent's checks
   passed, the combined change fails, nothing lands" is the clip and the
   product. Merge queues test combined state but know nothing about agents or
   ownership.
2. **Semantic ownership collisions across concurrent agent PRs** via Signal
   Core. Race Shield generalized from one machine to a repository.
3. **Exact-byte, stale-safe Apply with journaled recovery and
   ownership-scoped revert**: undo one agent's landing after later commits.
4. **Receipts as compliance artifacts**: who (agent, model, version, prompt
   hash), what (bytes, files, symbols), checks (commands, exit codes, on which
   digest), who approved, how to recover. Emit them as [in-toto](https://rywalker.com/research/in-toto) attestations so supply-chain tooling (Sigstore, gittuf, GitHub artifact attestations) can consume them. No standard predicate for AI authorship exists yet (the closest is a hackathon project, "Pedigree"); Pytxo can publish one.
5. **Neutral and local-first**, with an air-gap story for regulated teams.
6. **Per-run cost governor across vendors**: the wallet and meter code exists;
   repurpose it as a hard budget per landed change.

### 4.4 Complement, don't fight

Orca, Conductor, Superset, Emdash, the Codex app and Claude Code Projects
*produce* branches. Pytxo *lands* them. Publish "Run in Orca, land with Pytxo"
guides; ask Orca (MIT) and Superset for a "Land via Pytxo" action; list every
workspace tool as a supported input. Their 89k stars become Pytxo's funnel
instead of its competition.

### 4.5 Business model

| Tier | Who | Price (planning anchors) | Includes |
|---|---|---|---|
| **Free** | Individuals, public repos | $0 forever | Desktop, CLI, unlimited local gate, receipts kept locally |
| **Team** | Private repos, 2–50 devs | ~$20/dev/mo monthly, ~$16 annual | GitHub App, combined queue, hosted receipts 90 days, Slack, policy file |
| **Enterprise** | Platform/AppSec buyers | $40–60/dev or platform fee | Self-hosted gate and ledger, SSO/SCIM, retention, audit exports, policy-as-code, support |
| **Usage add-on** | Teams without CI | Metered | Hosted verification compute only where customers lack runners; otherwise none |

Anchors: CodeRabbit $24–30/dev, Mergify ~$21/seat, Superset $20/user,
Conductor Teams $60/user, Cursor Teams $40/user. Billing basis: developers
whose PRs the gate processed in the last 30 days (CodeRabbit and Mergify both
bill active contributors), so bots and idle seats are free.

Licensing: keep the core MIT (trust, adoption, auditability). Put the hosted
ledger and team coordination server under a source-available license (BSL or
ELv2, as Superset did) or keep it closed. Vibe Kanban's team features were
coordination, which teams did not need to pay for. Pytxo's team features are
landing evidence, which teams already pay for in the review layer.

Who pays and why: the engineering manager or platform lead accountable for
`main`. Pain: review load, incidents, "which agent did this." Artifact: a
receipt they can show a reviewer, a postmortem or an auditor.

### 4.6 The SaaS question, answered

The SaaS is the Ledger plus the GitHub App, not the desktop app. Open-core
plus hosted is the proven pattern for developer infrastructure, and hosted
services reach monetization faster than pure open core. Keep the local
product free and complete so that adoption never has a paywall in front of
it, and charge for the thing that only matters once more than one person is
accountable for a repository.

### 4.7 Funding and structure

- **Do not raise before ~50 weekly active users.** Nobody funds zero users,
  and a $22M-funded free competitor already exists in the old category.
- **Grants and fellowships that fit an under-18 solo founder** (verify
  eligibility and terms on each site): Emergent Ventures (Mercatus; applicants
  13+, rolling), [1517 Fund](https://www.1517fund.com/grants/) grants and seed, Z Fellows ($10k, one week, rolling, equity terms unpublished), GitHub Sponsors for the OSS core.
- **YC** (W27/S27) only with traction and the guardian/entity question
  resolved in advance.
- **Acquisition is a legitimate outcome, not a failure.** Graphite → Cursor,
  Continue → Cursor, Windsurf → Cognition, and Bloop's shutdown all happened
  in 12 months. GitHub, Cursor, Anthropic, OpenAI and the review-tool vendors
  all have a reason to want a neutral landing layer with provenance. Build it
  so that it is worth buying, and keep the option open.
- **Legal and payments** stay on the MBCZ/Dodo guardian structure in
  [[dodo-mor-integration]]; publish prices, Terms, Privacy and Refund pages
  before any checkout; code-sign the MSI early (a few hundred dollars a year
  buys a lot of trust).

### 4.8 Distribution: the part that has been missing

1. **Launch week (everything is ready).** Product Hunt pack, film v8, Show HN
   in Matt's words, Reddit in Matt's words, X clip. Then DM 20 people who
   complain about merging parallel agents' work or about Conductor having no
   Windows build. Sit on 5 real runs.
2. **Reposition the one sentence now.** Site and README lead with "lands
   agent work" and the Apply-gate clip (each agent passes, combined fails,
   nothing lands). That is a copy change, not a redesign.
3. **Public proof program.** Use Pytxo to land real PRs in popular
   open-source repositories and publish every receipt: "3 agents, 7 files,
   checks on the combined change, landed." Credible, shareable, and the
   receipts become the demo.
4. **Build in public, deliberately.** A solo teenage founder building Rust
   agent infrastructure with AI is a genuine distribution asset in long-form
   (YouTube essays, the pipeline approved 10 Oct, weekly "what agents landed
   this week"). Keep the age out of bios and payment-facing profiles, as the
   9 Oct account audit recommended; use the story where Matt controls the
   framing.
5. **Marketplaces.** GitHub Marketplace (the App), the official MCP registry
   (free, feeds downstream catalogs), Claude Code plugin marketplace, Codex and
   Cursor directories; Vanta/Drata integration listings once the Ledger exists.
6. **Ecosystem.** Compatibility guides for Orca, Conductor, Superset, Codex
   app, Claude Code Projects. Publish the receipt predicate as an open spec and
   invite CodeRabbit, Mergify and GitHub to consume it; the reference
   implementation of a standard is a durable position (Sigstore → Chainguard).
7. **Cadence.** A release every week, a receipt every week, a post every week.

### 4.9 Metrics and kill gates (planning targets, not forecasts)

| When | Continue if | Otherwise |
|---|---|---|
| 14 days after launch | ≥5 of 20 contacted developers run Pytxo twice unprompted | Change the message; no new features |
| 90 days | ≥500 installs, ≥50 weekly active, 3 teams running the gate in shadow mode, 1 asks about paying | Narrow to repository assurance or stop |
| 6 months | 10 paying teams (~$2k MRR), 1 enterprise pilot, 20 public receipts in OSS repos | Reassess wedge |
| 12 months | $10–20k MRR, or a serious acquisition conversation | Stop or consult |
| 24 months | Credible path to $1M ARR, or exit | — |

Second-run rate within two weeks is the one number to watch.

## 5. What not to do

- Do not build hosted sandboxes or cloud fleets (Conductor, Warp Oz, Cursor,
  Codex and Claude Projects own that, and it has COGS).
- Do not build macOS Desktop before the gate has users; the gate is
  platform-independent.
- Do not make "more agents supported" the headline; Orca has 27.
- Do not sell "hypervisor," "commit layer" or "effect contracts" to
  developers.
- Do not charge individuals or paywall any local feature.
- Do not chase enterprise-security buyers head-on; arrive from the developer
  workflow.
- Do not spend another month on UI before 100 users (9 Oct verdict).
- Do not weaken the fact-sheet discipline; the do-not-claim list is a moat
  against the "agent washing" Gartner describes.

## 6. Ninety-day sketch (sequence only; nothing here is implemented)

| Weeks | Focus | Approval gates |
|---|---|---|
| 1–2 | Launch; 20 DMs; 5 real runs; fix what breaks; one-sentence repositioning on site and README; compatibility guides | Publication, deploy |
| 3–6 | `pytxo gate` shadow mode as CLI + GitHub Action: ownership conflicts, combined-state checks, receipt v0 (JSON + PR comment); 3 design partners | — |
| 7–10 | GitHub App check run; symbol-level cross-PR collisions via Signal Core; combined-queue verification; MCP `land` tool; first OSS proof PRs | Marketplace listing |
| 11–13 | Team checkout through the existing Link/Dodo path; Ledger MVP (hosted receipts, retention); grant applications; code signing | Pricing, Dodo activation, legal pages |

The alignment program's Phase 0 (15 platform-team interviews) runs *inside*
this, not before it: the 20 DMs and 3 design partners are the interviews.

## 7. Moonshots (only after the H1 gates pass)

- **The receipt standard.** An open in-toto predicate for agent-authored
  changes, adopted by review tools and GitHub attestations, with Pytxo as the
  reference implementation and hosted verifier.
- **Effect contracts beyond the repo** ([[ADR-0036-effect-contract-commit-boundary]]).
  The PocketOS incident is a missing gate on a volume delete. Shadow adapters
  for Terraform plan/apply and Kubernetes first, read-only, with design
  partners, exactly as [[pytxo-commit-layer-alignment]] Phase 1 specifies.
- **Agent FinOps governor.** Hard per-run and per-landed-change budgets across
  vendors; the Uber story is the sales deck.
- **Evidence an insurer or auditor accepts.** Receipts mapped to SOC 2 change
  management and ISO 27001 controls; partnerships with compliance platforms.
- **A verifier ecosystem.** Third-party checks (security, licensing, data
  policy) attach to the gate. Platform optionality, not an early marketplace.

## 8. Risks and specific failure modes

| Risk | Specific scenario | Response |
|---|---|---|
| GitHub absorbs the gate | Agent HQ adds combined-queue verification with agent provenance and branch controls in one release | Neutrality (GitLab, Bitbucket, local, air-gap); symbol-level collisions; receipts as an open standard; local Apply and revert that a hosted service cannot offer |
| Vendor auth policy | Anthropic treats 8 parallel headless sessions as outside "ordinary individual usage" | BYOK API keys first-class; cost governor; gate on outputs, not on how agents ran |
| Zero audience persists | Launch produces <5 second runs | Change the message and the clip before features; proof program; stop if the 90-day gate fails |
| Solo operator | School, burnout, bus factor, five vendor integrations breaking weekly | Weekly small releases; automated acceptance already exists; recruit a maintainer from OSS contributors; keep vendor adapters thin |
| Under-18 legal | Contracts, payments, investor paperwork | Guardian/MBCZ entity for every external agreement; no age in public profiles |
| Trust | Unsigned installer, no SOC 2 | Code signing now; SOC 2 via a compliance platform once revenue exists |
| Claims drift | Marketing outruns the product | Fact sheet and do-not-claim list stay canonical |

## 9. Decisions only Matt can make

1. Adopt "the landing layer for agent-written changes" as the next 90-day
   focus, with the gate as the first new surface?
2. License boundary for the hosted ledger and team server: MIT, BSL/ELv2, or
   closed?
3. Team price point and the free-for-public-repos stance.
4. Whether and where to use the founder story in long-form content.
5. Which grants or fellowships to apply to this quarter.
6. Launch date, given time available for the first hours of HN and Product
   Hunt.

## Sources

Checked 2026-10-10. Primary where available; others are press, aggregators or
vendor pages and are marked in the text.

- Anthropic, [Claude Code legal and compliance](https://code.claude.com/docs/en/legal-and-compliance)
- Conductor [pricing](https://www.conductor.build/pricing); Tracxn [profile](https://tracxn.com/d/companies/conductor/__GDvVaFT0iP5pizAQSml7d0e-1TcgZIeanC09x60Mwto)
- Superset [pricing](https://superset.sh/pricing); Orca [site](https://www.onorca.dev/) and [repository](https://github.com/stablyai/orca); [T3 Code](https://github.com/pingdotgg/t3code); [Emdash](https://github.com/generalaction/emdash); [Composio Agent Orchestrator](https://github.com/ComposioHQ/agent-orchestrator); Vibe Kanban [shutdown notice](https://vibekanban.com/shutdown)
- OpenAI, [Introducing the Codex app](https://openai.com/index/introducing-the-codex-app/); GitHub, [Claude and Codex on Agent HQ](https://github.blog/news-insights/company-news/pick-your-agent-use-claude-and-codex-on-agent-hq/); VS Code, [multi-agent development](https://code.visualstudio.com/blogs/2026/02/05/multi-agent-development); Claude Code Projects relaunch coverage, [AIChief](https://aichief.com/news/claude-code-relaunches-projects-to-manage-multiple-ai-agents-in-the-cloud/)
- Cursor revenue, [Dealroom](https://dealroom.co/news/134107-cursor-tops-4b-annualized-revenue/) and [TechCrunch](https://techcrunch.com/2026/04/17/sources-cursor-in-talks-to-raise-2b-at-50b-valuation-as-enterprise-growth-surges/); Factory, [Metir AI](https://www.metirai.com/blog/factory-ai-200-million-5-billion-coding-agents-2026) and [pricing](https://www.factory.ai/pricing); Cognition, [VKTR](https://www.vktr.com/ai-platforms/cognition-ai-in-talks-to-raise-over-1-billion-at-40-billion-valuation/)
- CodeRabbit, [Series C](https://ecvc.wsgr.com/news/wilson-sonsini-advises-coderabbit-on-dollar143-million-series-c-funding), [Sacra](https://sacra.com/c/coderabbit/), [pricing guide](https://www.augmentcode.com/guides/coderabbit-pricing-costs-limits); Graphite, [Contrary Research](https://research.contrary.com/company/graphite); Mergify [pricing](https://mergify.com/pricing)
- Governance funding: [Fortune on Geordie](https://fortune.com/2026/05/28/geordie-security-governance-ai-agents/), [Calcalist on Rein](https://www.calcalistech.com/ctechnews/article/rkf52cnimg), [CVJ.AI](https://cvj.ai/briefing/finance-news/ai-agent-security-startups-raise-435m-as-governance-becomes-the-bottleneck/), [Q3 tracker](https://gravity.fast/blog/ai-agent-funding-tracker-q3-2026/), [Island and Cyera](https://valueaddvc.com/pulse/value-add-pulse-ai-security-funding-pace-2026); Microsoft [Entra Agent ID](https://learn.microsoft.com/en-za/entra/agent-id/whats-new-agent-id)
- Review bottleneck: [Codacy on CircleCI 2026](https://blog.codacy.com/ai-breaking-code-review-how-engineering-teams-survive-pr-bottleneck), [LeadDev](https://leaddev.com/ai/as-ai-helps-us-write-more-code-whos-catching-the-bugs), [LinearB summary](https://www.kunalganglani.com/blog/ai-coding-assistant-team-adoption), [Faros summary](https://votazz.co/2026/07/07/ai-code-review-bottleneck/)
- Stack Overflow 2026 survey coverage, [IT Brief](https://itbrief.co.uk/story/stack-overflow-survey-shows-developers-trust-ai-with-caveats); Windows share, [Stack Overflow 2025 via Windows Latest](https://windowslatest.com/2026/10/05/microsoft-distinguished-engineer-fires-back-at-anti-windows-11-rage-bait-says-more-developers-use-windows-than-linux-or-macos)
- Incidents: OECD AI incident records for [PocketOS](https://oecd.ai/en/incidents/2026-04-27-6153) and the [Terraform deletion](https://oecd.ai/en/incidents/2026-03-07-79ec); FinOps, [Unblocked](https://getunblocked.com/blog/finops-for-ai-coding/) and [Quali](https://www.quali.com/blog/the-token-debt-problem-why-enterprises-cant-control-ai-costs/)
- Gartner forecast coverage, [Outlook Business](https://www.outlookbusiness.com/deeptech/artificial-intelligence/over-40-of-agentic-ai-projects-will-be-scrapped-by-2027-says-gartner); EU AI Act delay, [Sidley](https://datamatters.sidley.com/2026/06/22/eu-lawmakers-reach-provisional-agreement-to-delay-key-eu-ai-act-obligations/)
- Provenance: [in-toto overview](https://rywalker.com/research/in-toto); [Pedigree hackathon project](https://lablab.ai/ai-hackathons/ibm-bob-hackathon/ctrlcats/pedigree); MCP registry state, [Towards AI](https://pub.towardsai.net/mcp-registries-in-mid-2026-one-upstream-won-2ce541b92036)
- Funding for young founders: [1517 Fund grants](https://www.1517fund.com/grants/); Emergent Ventures (Mercatus Center); Z Fellows (third-party [guide](https://hub.causo.ai/guides/how-to-apply-to-z-fellows-2026))
- Internal: `LAUNCH.md`, `CHECKPOINT.md`, `README.md`, `D:/marketing/campaigns/pytxo-public-beta-launch/fact-sheet.md`, `D:/marketing/channels/2026-10-09-account-audit.md`, GitHub API (stars, traffic, release assets), npm downloads API, 9 Oct 2026 session transcript (viability verdict and pitch)

Back: [[MOC-home]] · [[product-vision]] · [[positioning]]
