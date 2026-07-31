---
title: v1 provider authentication and onboarding research
slug: v1-provider-auth-onboarding-research
status: active
tags:
  - project
  - research
  - desktop
  - authentication
  - providers
audience:
  - human
  - agent
layer: orchestration
created: 2026-07-30
updated: 2026-07-30
related:
  - "[[ADR-0014-multi-provider-byok-catalog]]"
  - "[[permission-profile-engine]]"
  - "[[execution-domains]]"
  - "[[product-vision]]"
---

# v1 provider authentication and onboarding research

## Verdict

Pytxo should not build a universal “Sign in with ChatGPT, Claude, DeepSeek, and more” layer. Those names currently hide three different relationships:

1. **Pytxo account** — optional Pytxo Cloud or Ultra identity.
2. **Agent harness session** — an installed Codex, Claude Code, Cursor Agent, Gemini CLI, Copilot CLI, or OpenCode process authenticated by its vendor.
3. **Inference-provider credential** — an API key, workload identity, local model, or managed Pytxo transport used for model requests.

Conflating them would mislead users, widen Pytxo’s secret-handling surface, and violate some vendors’ documented rules. The honest v1 design is a local **auth broker and status surface**: Pytxo launches official CLI-owned login flows, reads only documented non-secret status, and injects only a deliberately selected provider credential into an execution child.

## Supported boundary

| Integration | Honest v1 action | Credential owner | Pytxo must not do |
|---|---|---|---|
| OpenAI Codex / ChatGPT | Use Codex App Server account RPCs for browser or device login | Codex | Capture ChatGPT credentials or operate Codex’s OAuth tokens itself |
| Anthropic Claude Code | Launch `claude auth login`; read `claude auth status` | Claude Code | Offer a Pytxo-owned “Sign in with Claude.ai” flow or reuse consumer OAuth |
| DeepSeek API | Configure an API-key environment reference | User / OS environment | Call it a login, store the key in Svelte, SQLite, TOML, logs, or command arguments |
| Gemini CLI | Launch the official CLI authentication flow | Gemini CLI | Harvest or reuse the cached Google OAuth credential |
| GitHub Copilot CLI | Launch `copilot login`; prefer ACP for structured execution | Copilot CLI / OS keychain | Read or copy its token store |
| Cursor Agent | Launch `cursor-agent login`; read its status command | Cursor Agent | Inspect its stored credential |
| OpenCode | Launch its provider-specific `/connect` experience | OpenCode | Inspect `auth.json` or present every provider method as Pytxo-supported |
| Aider | Pass an explicitly selected environment reference | User / OS environment | Discover and forward every credential inherited by Desktop |

## What the vendors officially support

### OpenAI Codex

Codex is the strongest candidate for a native-feeling Pytxo connection. The [Codex App Server authentication API](https://developers.openai.com/codex/app-server) exposes `account/read`, `account/login/start`, login completion and cancellation notifications, `account/logout`, and rate-limit reads. In ChatGPT-managed mode, **Codex owns the OAuth lifecycle**, including persistence and refresh; it supports browser and device-code flows. API-key mode is separate.

OpenAI’s [Codex authentication guide](https://developers.openai.com/codex/auth) also distinguishes subscription-based ChatGPT access from usage billed through an API key. Pytxo can therefore render “Connect Codex with ChatGPT” while the Codex process remains the credential owner. The experimental external-token mode is for a host that already owns ChatGPT authentication; Pytxo is not such a host and should not adopt it.

Codex may cache credentials in a local auth file or the operating-system credential store. Pytxo should encourage the keyring-backed Codex setting but treat storage as Codex’s responsibility.

### Anthropic Claude Code

Claude Code documents [`claude auth login`, `claude auth logout`, and `claude auth status`](https://code.claude.com/docs/en/cli-usage). Its [authentication documentation](https://code.claude.com/docs/en/authentication) covers Claude.ai or Console credentials, API keys, and supported cloud-provider identities. The CLI stores its own credentials using the OS keychain where supported or a protected local credential file.

The legal boundary matters more than the mechanics. Anthropic’s [legal and compliance documentation](https://code.claude.com/docs/en/legal-and-compliance) states that Claude.ai OAuth is intended for Anthropic’s own products and that third-party developers should use API-key or supported cloud-provider authentication. Pytxo must therefore **open Claude Code’s own sign-in**, not implement or proxy “Sign in with Claude.” For programmatic direct Anthropic access, Pytxo should use an API key or supported workload identity, consistent with the [Claude Platform authentication guide](https://platform.claude.com/docs/en/manage-claude/authentication).

### DeepSeek

DeepSeek’s [official API documentation](https://api-docs.deepseek.com/) documents Bearer API-key authentication and an OpenAI-compatible base URL. Its [chat completion reference](https://api-docs.deepseek.com/api/create-chat-completion/) does not document a consumer OAuth or desktop sign-in flow for third-party applications.

The v1 card must therefore say **“Add DeepSeek API key reference,” not “Sign in with DeepSeek.”** A key disclosed in conversation, a recording, a screenshot, or a log must be treated as exposed and revoked before it is used for testing.

### Other useful harnesses

- [Gemini CLI authentication](https://geminicli.com/docs/get-started/authentication/) supports browser-based Google authentication owned by the CLI, API keys, and Vertex credentials. Pytxo may launch that flow but must not harvest or repurpose the cached OAuth token.
- [GitHub Copilot CLI authentication](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli) supports its own OAuth/device flow, GitHub CLI credentials, and tokens for automation. Pytxo should launch `copilot login` and later prefer Copilot’s documented [Agent Client Protocol support](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/about-copilot-cli) for structured integration.
- [Cursor Agent authentication](https://docs.cursor.com/en/cli/reference/authentication) provides browser login plus `cursor-agent login`, `status`, and `logout`, as well as an API key for automation. Its credential remains Cursor-owned.
- [OpenCode provider documentation](https://opencode.ai/docs/providers/) exposes provider-specific `/connect` flows. Pytxo should treat OpenCode as an independently authenticated harness rather than reading its credential files.
- [Aider API-key guidance](https://aider.chat/docs/config/api-keys.html) is environment/configuration based. Pytxo should pass only the credential reference chosen for that run.

If Pytxo ever owns an OAuth client for **Pytxo’s own service**, [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252.html) requires a native application to use an external user-agent and PKCE. An embedded Tauri webview is not the sign-in surface.

## Current Pytxo gap

The existing registry in `crates/pytxo-core/src/ade_registry.rs` knows whether six harness binaries are installed, but it has no auth capability or session model. `list_ade_clis` exposes installation metadata, and Desktop consequently presents integrations as only installed or not installed.

The current BYOK screen and [[ADR-0014-multi-provider-byok-catalog]] have the correct narrow promise: show whether a known environment variable is set without becoming a general key vault. However, execution currently undermines that promise:

- the PTY backend inherits the Desktop process environment and removes only Pytxo’s Ultra session;
- the subprocess backend also inherits its parent environment;
- provider selection can remap one selected key, but it does not clear unrelated inherited credentials.

This means a child agent can receive unrelated secrets simply because they were present when Desktop launched. **Explicit child-environment construction is a v1 release blocker.**

Pytxo’s own optional account flow also needs hardening before it represents release-ready authentication. The current deep-link handler logs the complete callback URL, the bearer token travels in its query string, and JWT validation parses expiry without cryptographically verifying signature, issuer, or audience. Keyring storage and removal of the Pytxo session from children are good foundations, but they do not compensate for an exposed callback or unverified token.

## Recommended v1 architecture

### 1. Model auth as capabilities, not a universal provider state

Add an orchestration-owned `HarnessAuthAdapter` with safe metadata only:

```text
installed + version
auth state: signed_in | signed_out | expired | unknown
auth owner: vendor_cli | os_environment | cloud_identity | pytxo_proxy | none
supported intents: login | logout | recheck
method label + redacted account hint, only when the official status API returns it
minimum supported version + official documentation URL
```

The adapter never returns access tokens, refresh tokens, cookies, or credential-file paths to Svelte. Presentation requests an intent; the Rust control plane executes it.

Codex should use App Server JSON-RPC. Claude Code, Copilot, Cursor Agent, and Gemini should use their documented login and status surfaces. OpenCode and unknown harnesses should fall back to launching their vendor-owned interactive connect command without credential inspection.

### 2. Keep API-key onboarding separate

The provider screen should configure a **credential source**, not accept raw secrets:

- environment variable reference;
- supported cloud workload identity;
- local-model endpoint;
- Pytxo managed transport, when legitimately available.

For v1, preserve ADR-0014: no key text fields in Svelte and no secret persistence in SQLite or TOML. If Pytxo later becomes a key vault, that requires a new ADR, OS-keyring storage, Tauri-only secret ingress, redacted error paths, and a threat model.

### 3. Make the UI language precise

An integration card should expose:

- **Install** when the harness is absent;
- **Open vendor sign-in** when its official flow is available;
- **Recheck** for a non-billable status probe;
- **Use in Flow** after the harness is ready.

“Disconnect from Pytxo” and “Sign out of this vendor on this device” are different actions. Vendor logout can affect other terminals and must be an explicit, confirmed machine-wide action. Readiness should show two separate facts, for example: “Codex installed · ChatGPT connected” or “Aider installed · DeepSeek key reference missing.”

### 4. Put the auth broker outside execution domains

Login is a host control-plane action. It must not run from a repository working directory, load project-controlled configuration, or execute inside an agent sandbox. Auth status is telemetry; permission policy remains in orchestration.

At launch, construct each child environment from a documented minimal baseline, then add only:

1. variables required by the selected harness;
2. the single selected provider credential reference;
3. explicit run variables approved by policy.

Do not inherit the complete Desktop environment. CLI-owned sessions can remain available through the vendor’s documented keychain or profile mechanism without copying their values into Pytxo. DeepSpace should have no cloud credential injection; Orbit remains the default profile for ordinary local runs.

### 5. Repair Pytxo account authentication

Replace bearer-token deep links with an opaque, one-time authorization code bound to `state` and PKCE, opened in the system browser. Exchange the code in Rust, validate the returned token cryptographically against expected issuer and audience, remove full callback-URL logging, and sanitize all error and telemetry paths. The browser callback and agent-provider connection UI should never reveal a token.

## v1 security gates and acceptance criteria

Pytxo provider onboarding is release-ready only when:

- no Pytxo UI or IPC response handles vendor passwords, browser cookies, OAuth tokens, or raw provider keys;
- every vendor-login button delegates to a documented vendor-owned flow;
- Claude.ai and Gemini consumer OAuth are not presented as reusable Pytxo credentials;
- every provider label distinguishes CLI subscription login from metered API use;
- auth rechecks do not send a model request or create billable usage;
- child processes receive an explicit allowlisted environment, proven by a test with unrelated sentinel secrets;
- login processes start outside repository-controlled working directories;
- logs, screenshots, telemetry, diagnostics, and demo fixtures contain no secrets;
- Pytxo’s own callback uses external-browser authorization code + PKCE and validates signed tokens;
- logout scope is explained and confirmed;
- expired, revoked, missing, unsupported-version, offline, and unknown-status states have actionable UI copy;
- a real end-to-end test covers Codex App Server login/status/logout, Claude CLI-owned login/status, and a DeepSeek API-key reference without persisting or displaying the key.

This boundary produces a more credible v1 than a wide provider-login gallery. Pytxo becomes useful by coordinating already-trusted harnesses while preserving their security model—not by impersonating their account systems.
