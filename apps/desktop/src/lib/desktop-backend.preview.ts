/**
 * Fixture data and backend implementation used ONLY outside a real Tauri
 * runtime (browser dev server, Storybook, Playwright). `createDesktopBackend`
 * in `desktop-backend.ts` gates this behind a strict `__TAURI_INTERNALS__`
 * check, so a packaged Tauri build can never silently fall back to this data
 * — a failed IPC call surfaces as a real error instead.
 */
import type { DesktopBackend, DesktopSnapshot } from "./desktop-backend";
import { DESKTOP_BETA_MAX_WORKERS, isBetaAde } from "./ade-status";
import type { AdeCliStatusDto, DesktopChangedEvent, DomainChangeDto, FlowDraftInput, FlowDraftRecord, FlowPlan, PermissionEnforcementReceipt, PreparedContentChunkDto, PreparedRunManifest, ProposedHostedAdvisorPacketPreview, ProviderStatusDto, ReviewedDemandFacts, ReviewedHostedAdvisorPacketPreview, RoutingHostedGrantStatus, RoutedAdvisorConsentStatus, RoutedAdvisorPacketPreview, RoutingDisplaySummary, RunApplyError, RunApplyManifest, RunReviewDto, VoiceProgressEvent, VoiceSessionDto } from "./types";

const previewSignalMark = Uint8Array.from({ length: 300 }, (_, index) => index % 251);
previewSignalMark.set([0x00, 0xff, 0x50, 0x4e, 0x47], 0);
previewSignalMark.set([0xde, 0xad, 0xbe, 0xef], previewSignalMark.length - 4);

const previewReviewContent: Record<string, { before?: string | Uint8Array; after?: string | Uint8Array }> = {
  "crates/pytxo-signal/src/lib.rs": {
    before: "pub fn skeleton(source: &str) -> String {\n    source.to_owned()\n}\n",
    after: "pub fn skeleton(source: &str) -> String {\n    parse(source).structural_skeleton()\n}\n",
  },
  "crates/pytxo-signal/tests/skeleton.rs": {
    after: "#[test]\nfn keeps_public_shape() {\n    assert!(skeleton(\"pub fn run() {}\").contains(\"run\"));\n}\n",
  },
  "crates/pytxo-signal/src/legacy.rs": {
    before: "pub fn legacy_raw_context(source: &str) -> String { source.to_owned() }\n",
  },
  "assets/signal-mark.bin": {
    after: previewSignalMark,
  },
};

// Opt-in substantial-content fixture for browser readability/scroll acceptance.
// Never available through native IPC; no execution or verification evidence.
function reviewContentFixture(): typeof previewReviewContent {
  if (localStorage.getItem("pytxo-preview-review-substantial-v1") !== "true") return previewReviewContent;
  const cases = Array.from({ length: 36 }, (_, index) =>
    `#[test]\nfn preserves_public_shape_${index}() {\n    let source = "pub fn entry_${index}() {}";\n    let result = skeleton(source);\n    assert!(result.contains("entry_${index}"));\n}\n`).join("\n");
  const file = previewReviewContent["crates/pytxo-signal/src/lib.rs"];
  return { ...previewReviewContent, "crates/pytxo-signal/src/lib.rs": {
    before: `${file.before}\n// Recorded source comparison fixture\n${cases}`,
    after: `${file.after}\n// Recorded source comparison fixture\n${cases}`,
  } };
}

function previewBytes(value: string | Uint8Array): Uint8Array {
  return typeof value === "string" ? new TextEncoder().encode(value) : value;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export const previewSnapshot: DesktopSnapshot = {
  domains: [
    {
      domain_id: "pytxo",
      repo_root: "C:/dev/pytxo",
      db_path: "~/.pytxo/domains/pytxo.db",
      project_id: null,
      status: "active",
      updated_at: new Date().toISOString(),
      active_runs: 2,
      latest_run_status: "running",
      latest_started_at: new Date().toISOString(),
      hitl_pending: 1,
      is_available: true,
      is_temporary: false,
    },
    {
      domain_id: "signal-lab",
      repo_root: "C:/dev/signal-lab",
      db_path: "~/.pytxo/domains/signal-lab.db",
      project_id: null,
      status: "healthy",
      updated_at: new Date().toISOString(),
      active_runs: 0,
      latest_run_status: "completed",
      latest_started_at: new Date().toISOString(),
      hitl_pending: 1,
      is_available: true,
      is_temporary: false,
    },
  ],
  runs: [
    {
      id: "run-8f2c",
      domain_id: "pytxo",
      status: "running",
      repo_root: "C:/dev/pytxo",
      started_at: new Date().toISOString(),
      estimated_cost_usd: 0.42,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "projfs",
      apply_status: "pending",
      applied_at: null,
      prepared_digest: null,
      prepared_at: null,
      last_apply_error: null,
      recovery_state: null,
      routing_revision: null,
    },
    {
      id: "run-71ad",
      domain_id: "signal-lab",
      status: "completed",
      repo_root: "C:/dev/signal-lab",
      started_at: new Date(Date.now() - 3_600_000).toISOString(),
      estimated_cost_usd: 0.18,
      permission_profile: "orbit",
      isolation_mode: "copy_on_write",
      isolation_backend: "git_worktree",
      apply_status: "ready",
      applied_at: null,
      prepared_digest: "pkg-71ad-immutable",
      prepared_at: "2026-08-01T02:15:00Z",
      last_apply_error: null,
      recovery_state: null,
      routing_revision: null,
    },
  ],
  agents: [
    { id: "architect", domain_id: "pytxo", run_id: "run-8f2c", task_id: "plan", wave: 0, status: "completed", exit_code: 0, root_id: null },
    { id: "desktop", domain_id: "pytxo", run_id: "run-8f2c", task_id: "ui", wave: 1, status: "running", exit_code: null, root_id: null },
    { id: "verification", domain_id: "pytxo", run_id: "run-8f2c", task_id: "tests", wave: 2, status: "queued", exit_code: null, root_id: null },
  ],
  approvals: [
    {
      id: "approval-1",
      agent_key: "run-8f2c:desktop",
      run_id: "run-8f2c",
      agent_id: "desktop",
      action: "blast.flush",
      reason: "commit agent workspace to repo root",
      created_at_ms: String(Date.now()),
      domain_id: "pytxo",
    },
    {
      id: "approval-2",
      agent_key: "run-71ad:agent-1",
      run_id: "run-71ad",
      agent_id: "agent-1",
      action: "net.egress",
      reason: "network fetch detected in agent command",
      created_at_ms: String(Date.now() - 90_000),
      domain_id: "signal-lab",
    },
  ],
  fleets: [],
  diagnostics: [],
  error: null,
};

// Opt-in browser fixture only. It represents Store's whitelisted read model,
// never an actual routing or worker execution receipt.
const previewRoutingSummary: RoutingDisplaySummary = {
  domain_id: "pytxo", run_id: "run-8f2c", routing_revision: "3", mode: "shadow", cancelled: false,
  tasks: [
    { task_id: "plan", state: "succeeded", dependency_task_ids: [], current_attempt_id: "route-plan-1", winning_attempt_id: "route-plan-1", last_decision: { selection: { kind: "selected", role: "everyday" }, reason: "mechanical_everyday", advice_status: "shadow_recorded" }, pre_admission: null, attempts: [
      { attempt_id: "route-plan-1", agent_id: "architect", ordinal: 1, predecessor_id: null, state: "passed", role: "everyday", decision: { selection: { kind: "selected", role: "everyday" }, reason: "mechanical_everyday", advice_status: "shadow_recorded" }, profile_id: "codex-everyday", harness_id: "codex", billing_mode: "subscription", handoff_referenced: false, sealed_output_recorded: true, checks_receipt_recorded: true, usage_status: "unknown", ownership_released: true, admitted_at_ms: "1780000000000", updated_at_ms: "1780000001000" },
    ] },
    { task_id: "ui", state: "active", dependency_task_ids: ["plan"], current_attempt_id: "route-ui-2", winning_attempt_id: null, last_decision: { selection: { kind: "selected", role: "strong" }, reason: "strong_repair", advice_status: "rules_fallback" }, pre_admission: { ordinal: 2, decision: { selection: { kind: "selected", role: "strong" }, reason: "strong_repair", advice_status: "rules_fallback" }, outcome: { kind: "not_admitted", stage: "capacity_unavailable" } }, attempts: [
      { attempt_id: "route-ui-1", agent_id: "desktop-1", ordinal: 1, predecessor_id: null, state: "failed", role: "everyday", decision: { selection: { kind: "selected", role: "everyday" }, reason: "mechanical_everyday", advice_status: "shadow_recorded" }, profile_id: "codex-everyday", harness_id: "codex", billing_mode: "subscription", handoff_referenced: true, sealed_output_recorded: true, checks_receipt_recorded: true, usage_status: "known", ownership_released: true, admitted_at_ms: "1780000002000", updated_at_ms: "1780000003000" },
      { attempt_id: "route-ui-2", agent_id: "desktop-2", ordinal: 2, predecessor_id: "route-ui-1", state: "admitted", role: "strong", decision: { selection: { kind: "selected", role: "strong" }, reason: "strong_repair", advice_status: "rules_fallback" }, profile_id: "codex-strong", harness_id: "codex", billing_mode: "subscription", handoff_referenced: true, sealed_output_recorded: false, checks_receipt_recorded: false, usage_status: "unreported", ownership_released: false, admitted_at_ms: "1780000004000", updated_at_ms: "1780000004000" },
    ] },
    { task_id: "tests", state: "ready", dependency_task_ids: [], current_attempt_id: null, winning_attempt_id: null, last_decision: null, pre_admission: { ordinal: 1, decision: { selection: { kind: "selected", role: "everyday" }, reason: "mechanical_everyday", advice_status: "not_used" }, outcome: { kind: "not_admitted", stage: "capacity_unavailable" } }, attempts: [] },
  ],
};

/** Browser fixture for a mixed-CLI fleet run (`pytxo-preview-fleet-v1`). Illustrative, not a recorded run. */
const fleetTasks = [
  { task_id: "model", cli: "codex", name: "OpenAI Codex", paths: ["src/model.mjs", "test/model.test.mjs"], wave: 0, depends_on: [] as string[], status: "completed", lines: ["$ node --test test/model.test.mjs", "filters by status (2.1ms)", "search is case-insensitive (0.9ms)", "composes search and status (1.2ms)", "pass 11 · fail 0"] },
  { task_id: "dark-mode", cli: "claude", name: "Claude Code", paths: ["src/style.css", "src/app.js"], wave: 0, depends_on: [] as string[], status: "running", lines: ["Update(src/style.css) · 38 lines of dark tokens", "Update(src/app.js) · follows prefers-color-scheme", "Update(src/index.html) · theme toggle with aria-pressed", "Checking contrast of muted text"] },
  { task_id: "spanish", cli: "cursor", name: "Cursor Agent", paths: ["src/i18n/", "src/index.html"], wave: 0, depends_on: [] as string[], status: "running", lines: ["Creating src/i18n/es.json · 48 strings", "Wiring t() in src/index.html · 12 labels", "Adding language switch es / en"] },
  { task_id: "filter-bar", cli: "opencode", name: "OpenCode", paths: ["src/components/"], wave: 0, depends_on: [] as string[], status: "running", lines: ["src/components/filter-bar.js", "Empty state with Clear filters", "Status chips: All · Open · Done"] },
  { task_id: "validation", cli: "codex", name: "OpenAI Codex", paths: ["src/app.js", "test/validation.test.mjs"], wave: 1, depends_on: ["dark-mode"], status: null, lines: [] as string[] },
  { task_id: "readme", cli: "agy", name: "Antigravity", paths: ["README.md"], wave: 2, depends_on: ["model", "dark-mode", "spanish", "filter-bar", "validation"], status: null, lines: [] as string[] },
];

const previewAdeClis: AdeCliStatusDto[] = [
  { id: "codex", display_name: "OpenAI Codex", default_cmd: "codex exec --sandbox workspace-write", installed: true, auth_state: "signed_in", auth_label: "ChatGPT connected", auth_owner: "Codex", login_supported: true, login_label: "Connect with ChatGPT", docs_url: "https://developers.openai.com/codex/auth", detail: "Codex owns the browser session, token storage, and refresh." },
  { id: "claude", display_name: "Claude Code", default_cmd: "claude -p --permission-mode acceptEdits", installed: true, auth_state: "signed_in", auth_label: "Claude account connected", auth_owner: "Claude Code", login_supported: true, login_label: "Open Claude Code sign-in", docs_url: "https://code.claude.com/docs/en/authentication", detail: "Pytxo opens Claude Code's official sign-in and never receives its token." },
  { id: "cursor", display_name: "Cursor Agent", default_cmd: "cursor-agent -p --trust --output-format text", installed: true, auth_state: "signed_in", auth_label: "Cursor account connected", auth_owner: "Cursor Agent", login_supported: true, login_label: "Open Cursor sign-in", docs_url: "https://docs.cursor.com/en/cli/reference/authentication", detail: "Cursor Agent keeps its account credential outside Pytxo." },
  { id: "opencode", display_name: "OpenCode", default_cmd: "opencode run", installed: true, auth_state: "signed_out", auth_label: "No OpenCode provider connected", auth_owner: "OpenCode", login_supported: true, login_label: "Connect an OpenCode provider", docs_url: "https://opencode.ai/docs/providers/", detail: "Provider-specific credentials remain owned by OpenCode." },
  { id: "gemini", display_name: "Gemini CLI", default_cmd: "gemini --skip-trust -p", installed: true, auth_state: "vendor_managed", auth_label: "Authentication managed by Gemini CLI", auth_owner: "Gemini CLI", login_supported: true, login_label: "Open Gemini authentication", docs_url: "https://geminicli.com/docs/get-started/authentication/", detail: "Gemini CLI owns Google OAuth; Pytxo does not reuse its cached token." },
  { id: "copilot", display_name: "GitHub Copilot CLI", default_cmd: "copilot -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Copilot CLI", login_supported: true, login_label: "Open GitHub sign-in", docs_url: "https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli", detail: "Copilot CLI owns the GitHub device flow and stores its token in the OS keychain." },
  { id: "agy", display_name: "Antigravity", default_cmd: "agy --mode accept-edits -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Antigravity", login_supported: false, login_label: null, docs_url: "https://antigravity.google/docs/cli/headless/", detail: "Authentication and model access stay inside Antigravity." },
  { id: "aider", display_name: "Aider", default_cmd: "aider --message", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Pytxo run policy", login_supported: false, login_label: null, docs_url: "https://aider.chat/docs/config/api-keys.html", detail: "Choose one explicit BYOK credential for the run; unrelated keys stay hidden." },
  { id: "grok", display_name: "Grok Build", default_cmd: "grok --no-auto-update -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Grok Build", login_supported: true, login_label: "Open Grok Build sign-in", docs_url: "https://docs.x.ai/build/overview", detail: "Grok Build owns browser sign-in or XAI_API_KEY; Pytxo never reads its session." },
  { id: "droid", display_name: "Factory Droid", default_cmd: "droid exec --auto low", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Factory Droid", login_supported: false, login_label: null, docs_url: "https://docs.factory.ai/droid-exec/overview", detail: "Factory Droid owns its API credential and applies its low-autonomy policy inside Pytxo isolation." },
  { id: "cline", display_name: "Cline CLI", default_cmd: "cline --json", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Cline CLI", login_supported: true, login_label: "Open Cline authentication", docs_url: "https://docs.cline.bot/usage/cli-overview", detail: "Cline owns provider authentication and emits structured output for the isolated run." },
  { id: "goose", display_name: "Goose", default_cmd: "goose run --no-session -t", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Goose", login_supported: false, login_label: null, docs_url: "https://block.github.io/goose/docs/guides/goose-cli-commands/", detail: "Goose keeps its selected model provider and credentials in its own configuration." },
  { id: "qwen", display_name: "Qwen Code", default_cmd: "qwen -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Qwen Code", login_supported: false, login_label: null, docs_url: "https://github.com/QwenLM/qwen-code/blob/main/docs/users/features/headless.md", detail: "Detected on PATH, but write execution stays disabled until Qwen approval modes map to Pytxo permission profiles." },
  { id: "kimi", display_name: "Kimi Code CLI", default_cmd: "kimi -p", installed: false, auth_state: "not_installed", auth_label: "Not installed", auth_owner: "Kimi Code CLI", login_supported: true, login_label: "Open Kimi device sign-in", docs_url: "https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/reference/kimi-command.md", detail: "Kimi Code CLI owns its device login and applies its non-interactive permission policy." },
];

function previewAdeState(): AdeCliStatusDto[] {
  const state = typeof localStorage !== "undefined" ? localStorage.getItem("pytxo-preview-ade-state-v1") : null;
  if (state === "detection-error") throw new Error("Agent detection failed. Check again to retry.");
  if (!["codex-only", "no-agents", "codex-signed-out", "codex-unknown"].includes(state ?? "")) return structuredClone(previewAdeClis);
  return previewAdeClis.map((cli) => cli.id === "codex" && state !== "no-agents" ? {
    ...cli,
    ...(state === "codex-signed-out" ? { auth_state: "signed_out" as const, auth_label: "Codex sign-in required" } : {}),
    ...(state === "codex-unknown" ? { auth_state: "unknown" as const, auth_label: "Codex sign-in could not be verified" } : {}),
  } : {
    ...cli,
    installed: false,
    auth_state: "not_installed" as const,
    auth_label: "Not installed",
  });
}

const previewProviders: ProviderStatusDto[] = [
  { id: "deepseek", name: "DeepSeek", api_key_env: "DEEPSEEK_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openrouter", name: "OpenRouter", api_key_env: "OPENROUTER_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "openai", name: "OpenAI", api_key_env: "OPENAI_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
  { id: "anthropic", name: "Anthropic", api_key_env: "ANTHROPIC_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "gemini", name: "Google Gemini", api_key_env: "GOOGLE_API_KEY", key_configured: false, openai_compatible: false, builtin: true },
  { id: "mistral", name: "Mistral", api_key_env: "MISTRAL_API_KEY", key_configured: false, openai_compatible: true, builtin: true },
];

const previewReceipt: PermissionEnforcementReceipt = {
  requested_profile: "orbit",
  effective_profile: "orbit",
  execution_domain: "C:/dev/signal-lab",
  workspace_isolation: {
    status: "enforced",
    mechanism: "git-worktree",
    detail: "Agent mutations remain outside the primary repository until run-level Apply.",
  },
  host_filesystem_boundary: {
    status: "advisory",
    mechanism: "child-cwd-and-policy-gates",
    detail: "Workspace cwd and Pytxo-mediated path gates are active; syscall sandboxing is not.",
  },
  network: {
    status: "advisory",
    mechanism: "spawn-command-policy",
    detail: "Known network commands are denied; arbitrary child socket syscalls are not intercepted.",
  },
  apply_boundary: {
    status: "enforced",
    mechanism: "reviewed-run-atomic-apply",
    detail: "All claimed run changes are validated and in-process write failures roll back as one guarded operation.",
  },
};

/**
 * Preview-only fixture that exercises every `EnforcementSurface.status` and
 * `RunApplyAttempt.outcome` value in one screen. Real runs rarely produce all
 * eight at once, but each has to render a distinct, colour-independent
 * affordance, so the state matrix is asserted against this fixture rather than
 * against whatever a live run happens to report.
 */
const STATE_MATRIX_KEY = "pytxo-preview-state-matrix-v1";

const previewMatrixReceipt: PermissionEnforcementReceipt = {
  requested_profile: "galaxy",
  effective_profile: "orbit",
  execution_domain: "C:/dev/signal-lab",
  workspace_isolation: {
    status: "enforced",
    mechanism: "git-worktree",
    detail: "Agent mutations remain outside the primary repository until run-level Apply.",
  },
  host_filesystem_boundary: {
    status: "advisory",
    mechanism: "child-cwd-and-policy-gates",
    detail: "Workspace cwd and Pytxo-mediated path gates are active; syscall sandboxing is not.",
  },
  network: {
    status: "unavailable",
    mechanism: "not-reported-on-this-platform",
    detail: "No network enforcement evidence was recorded for this run.",
  },
  apply_boundary: {
    status: "bypassed",
    mechanism: "operator-override",
    detail: "The reviewed-apply boundary was explicitly disabled for this run.",
  },
};

function stateMatrixRequested(): boolean {
  return typeof localStorage !== "undefined" && localStorage.getItem(STATE_MATRIX_KEY) === "1";
}

const previewCompletedAgents = [
  { id: "run-71ad:agent-0", domain_id: "signal-lab", run_id: "run-71ad", task_id: "signal-core", wave: 0, status: "completed", exit_code: 0, root_id: null },
  { id: "run-71ad:agent-1", domain_id: "signal-lab", run_id: "run-71ad", task_id: "contract-tests", wave: 1, status: "completed", exit_code: 0, root_id: null },
];

export class PreviewDesktopBackend implements DesktopBackend {
  private readonly snapshot = structuredClone(previewSnapshot);
  private readonly voiceDevices = new Map<string, string>();
  private readonly cancelledVoiceSessions = new Set<string>();
  private readonly activeVoiceSessions = new Set<string>();
  private readonly domainListeners = new Set<(event: DesktopChangedEvent) => void>();
  private readonly changes: DomainChangeDto[] = [];
  private readonly recoveredRuns = new Set<string>();
  private shadowConsent: RoutedAdvisorConsentStatus = { domain_id: "signal-lab", revision: 0, enabled: false, current_scope: false, recipient_identity: "pytxo-local-advisor-fixture/no-network/v1", updated_at_ms: 0 };
  private readonly shadowConsentStorageKey = "pytxo-preview-shadow-grant-state-v1";
  private sequence = 0;

  constructor() {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-routing-summary-v1") === "1") {
      this.snapshot.runs[0].routing_revision = "3";
      if (localStorage.getItem("pytxo-preview-routed-retry-workers-v1") === "1") {
        this.snapshot.agents = this.snapshot.agents.flatMap((agent) => agent.id === "desktop"
          ? [{ ...agent, id: "desktop-1", status: "failed", exit_code: 1 }, { ...agent, id: "desktop-2", status: "running", exit_code: null }]
          : [agent]);
      }
      if (localStorage.getItem("pytxo-preview-routing-cancelled-v1") === "1") {
        this.snapshot.runs[0].status = "cancelled";
      }
      if (localStorage.getItem("pytxo-preview-routing-same-id-v1") === "1") {
        this.snapshot.runs.push({ ...this.snapshot.runs[0], domain_id: "signal-lab", repo_root: "C:/dev/signal-lab" });
      }
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-routing-revision-error-v1") === "1") {
      this.snapshot.runs[0].routing_revision = null;
      this.snapshot.diagnostics.push({ domain_id: "pytxo", run_id: "run-8f2c", stage: "routing_revision", message: "Browser fixture: routing revision read failed" });
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-many-runs-v1") === "1") {
      this.snapshot.runs.push(...Array.from({ length: 7 }, (_, index) => ({ ...this.snapshot.runs[0], id: `bb3355c8-330d-466e-8986-46cebc634c5${index}`, status: "completed" })));
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-legacy-retry-v1") === "1") {
      Object.assign(this.snapshot.runs[0], {
        status: "completed", apply_status: "applied", applied_at: "2026-09-19T21:00:00Z", recovery_state: "rolled_back",
        last_apply_error: { at: "2026-09-19T20:00:00Z", code: "interrupted_apply", message: "The interrupted Apply was rolled back.", attempt_id: "attempt-preview-1", rollback_confirmed: true },
      });
      this.snapshot.approvals = this.snapshot.approvals.filter(item => item.run_id !== "run-8f2c");
      this.snapshot.agents = this.snapshot.agents.map(agent => ({ ...agent, status: "completed", exit_code: 0 }));
      if (localStorage.getItem("pytxo-preview-newer-apply-error-v1") === "1") {
        this.snapshot.runs[0].recovery_state = null;
        this.snapshot.runs[0].last_apply_error!.at = "2026-09-19T22:00:00Z";
      }
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-approval-scope-v1") === "foreign-run") {
      const approval = this.snapshot.approvals.find((item) => item.run_id === "run-8f2c");
      if (approval) {
        approval.run_id = "run-other";
        approval.agent_key = "run-other:desktop";
      }
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-run-state-v1") === "review_failed") {
      Object.assign(this.snapshot.runs[0], {
        status: "failed", apply_status: "review_failed", prepared_digest: null, prepared_at: null,
        last_apply_error: { at: "2026-09-07T01:15:56Z", code: "review_preparation_failed", message: "runner: agent agent-0 edited test/risk-policy.test.mjs outside declared claims", attempt_id: null, rollback_confirmed: false },
      });
      this.snapshot.agents = this.snapshot.agents.map((agent) => ({ ...agent, status: "completed", exit_code: 0 }));
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-run-state-v1") === "starting") {
      this.snapshot.runs[0].status = "starting";
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-run-state-v1") === "agent_failed") {
      this.snapshot.runs[0].status = "failed";
      this.snapshot.approvals = this.snapshot.approvals.filter(item => item.run_id !== this.snapshot.runs[0].id);
      this.snapshot.agents = this.snapshot.agents.map((agent) => agent.id === "desktop" ? { ...agent, status: "failed", exit_code: 1 } : agent);
    }
    const nativeAgentFixture = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-native-agent-ids-v1");
    if (nativeAgentFixture === "switch") {
      this.snapshot.runs.push({ ...this.snapshot.runs[0], id: "run-other", status: "completed" });
      this.snapshot.agents.push(...this.snapshot.agents.map((agent) => ({ ...agent, run_id: "run-other" })));
    }
    if (nativeAgentFixture === "1" || nativeAgentFixture === "switch") {
      this.snapshot.agents = this.snapshot.agents.map((agent) => ({ ...agent, id: `${agent.run_id}:${agent.id}` }));
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-fleet-v1") === "1") {
      const run = this.snapshot.runs[0];
      this.snapshot.approvals = this.snapshot.approvals.filter(item => item.run_id !== run.id);
      this.snapshot.agents = [
        ...this.snapshot.agents.filter(agent => agent.run_id !== run.id),
        ...fleetTasks.filter(task => task.status).map(task => ({ id: `${run.id}:fleet-${task.task_id}`, domain_id: run.domain_id, run_id: run.id, task_id: task.task_id, wave: task.wave, status: task.status!, exit_code: task.status === "completed" ? 0 : null, root_id: null, launcher: { id: task.cli, display_name: task.name } })),
      ];
    }
    if (localStorage.getItem("pytxo-preview-agent-identity-v1") === "1") {
      this.snapshot.agents[0] = { ...this.snapshot.agents[0], launcher: { id: "codex", display_name: "OpenAI Codex" }, workspace_path: "C:/browser-fixture/isolated/worker" };
    }
    const requested =
      typeof localStorage === "undefined"
        ? null
        : localStorage.getItem("pytxo-preview-review-state-v1");
    const reviewRun = this.snapshot.runs.find((run) => run.id === "run-71ad");
    if (reviewRun && requested) {
      reviewRun.apply_status =
        requested === "recovered" ? "ready" : requested;
    }
  }

  private emitChange(domainId: string, entityKind: string, entityId: string) {
    const event = { domain_id: domainId, entity_kind: entityKind, entity_id: entityId };
    this.sequence += 1;
    this.changes.push({
      sequence: this.sequence,
      entity_kind: entityKind,
      entity_id: entityId,
      changed_at: new Date().toISOString(),
    });
    for (const callback of this.domainListeners) callback(event);
  }

  private resolveApproval(requestId: string) {
    const resolved = this.snapshot.approvals.find((approval) => approval.id === requestId);
    this.snapshot.approvals = this.snapshot.approvals.filter((approval) => approval.id !== requestId);
    if (!resolved?.domain_id) return;
    const domain = this.snapshot.domains.find((item) => item.domain_id === resolved.domain_id);
    if (domain) {
      domain.hitl_pending = this.snapshot.approvals.filter(
        (approval) => approval.domain_id === domain.domain_id,
      ).length;
    }
  }

  async loadSnapshot(opts: { includeAgents?: boolean } = {}) {
    if (localStorage.getItem("pytxo-preview-observe-polls-v1") === "1") {
      const reads = Number(localStorage.getItem("pytxo-preview-snapshot-reads-v1") ?? "0");
      localStorage.setItem("pytxo-preview-snapshot-reads-v1", String(reads + 1));
    }
    const snapshot = structuredClone(this.snapshot);
    // Match native IPC: omitted agent reads return no agent rows.
    if (opts.includeAgents === false) snapshot.agents = [];
    if (opts.includeAgents === false && localStorage.getItem("pytxo-preview-delayed-light-snapshot-v1") === "1") {
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    // Deliberately awkward browser-only data for responsive acceptance.
    const layoutFixture = localStorage.getItem("pytxo-preview-layout-fixture-v1");
    if (layoutFixture === "long") {
      snapshot.domains[0].repo_root = `C:/workspaces/${"a-long-folder/".repeat(8)}commit-boundary-20260901-160000-936`;
      snapshot.runs.push(...Array.from({ length: 40 }, (_, index) => ({ ...snapshot.runs[1], id: `history-fixture-${index}` })));
    }
    if (localStorage.getItem("pytxo-preview-history-empty-v1") === "1") snapshot.runs = [];
    return snapshot;
  }
  async approve(requestId: string) {
    this.resolveApproval(requestId);
    this.emitChange("pytxo", "approval", requestId);
  }
  async deny(requestId: string) {
    this.resolveApproval(requestId);
    this.emitChange("pytxo", "approval", requestId);
  }
  async stopRun(runId: string, domainId: string) {
    const domain = this.snapshot.domains.find((item) => item.domain_id === domainId);
    if (!domain) throw new Error(`Execution domain not found: ${domainId}`);
    const active = this.snapshot.runs.find(
      (run) =>
        run.repo_root === domain.repo_root &&
        ["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    );
    if (!active || active.id !== runId) {
      throw new Error(
        `Refusing to stop ${runId}: ${active ? `${active.id} is now active` : "no run is active"} in ${domainId}`,
      );
    }
    active.status = "cancelled";
    this.snapshot.agents = this.snapshot.agents.filter((agent) => agent.run_id !== runId);
    domain.active_runs = this.snapshot.runs.filter(
      (run) =>
        run.repo_root === domain.repo_root &&
        ["starting", "running", "pending", "dispatching", "active"].includes(run.status.toLowerCase()),
    ).length;
    domain.latest_run_status = active.status;
    this.emitChange(domainId, "run", runId);
  }
  async forgetDomain(domainId: string) {
    this.snapshot.domains = this.snapshot.domains.filter((domain) => domain.domain_id !== domainId);
  }
  async listAdeClis() {
    return previewAdeState();
  }
  async startAdeLogin(id: string) {
    const target = previewAdeState().find((item) => item.id === id);
    if (!target?.installed) throw new Error(`${target?.display_name ?? id} is not installed or is not on PATH.`);
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-login-error-v1") === "true") {
      throw new Error("Could not open vendor sign-in. Try again.");
    }
    return { id, message: `${target.display_name} sign-in opened. Finish the vendor flow, then recheck.` };
  }
  async listProviders() {
    return structuredClone(previewProviders);
  }
  async previewFlow(input: FlowDraftInput): Promise<FlowPlan> {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-flow-delay-v1") === "1") {
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-flow-error-v1") === "1") {
      throw new Error("Preview failed while checking the selected workspace. Fix the workspace issue, then build a new plan.");
    }
    const detected = previewAdeState();
    const requested = detected.find((cli) => cli.id === input.ade_id) ?? null;
    const available = !!requested && requested.installed && ["signed_in", "vendor_managed", "not_applicable"].includes(requested.auth_state);
    const omitVerification = typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-flow-verification-v1") === "none";
    const tasks = [
      { id: "desktop-flow", agent: input.ade_id ?? "codex", prompt: `Implement the Desktop slice of: ${input.mission_text}`, paths: ["apps/desktop/src/components/desktop2/FlowScreen.svelte"], dependencies: [], root: null, verify: ["npm run check"] },
      { id: "orchestration-flow", agent: "codex", prompt: `Implement the orchestration slice of: ${input.mission_text}`, paths: ["crates/pytxo-orchestrate/src/flow.rs"], dependencies: [], root: null, verify: ["cargo test -p pytxo-orchestrate"] },
      { id: "contract-tests", agent: "codex", prompt: `Verify the reviewed Flow contract for: ${input.mission_text}`, paths: ["crates/pytxo-orchestrate/tests/flow_mission.rs"], dependencies: ["desktop-flow", "orchestration-flow"], root: null, verify: ["cargo test -p pytxo-orchestrate"] },
    ];
    const max_workers = input.max_workers ?? 1;
    // Mirrors Core: round-robin over the selected CLIs, explicit per-task choices win.
    const selected = input.ade_ids?.length ? input.ade_ids : input.ade_id ? [input.ade_id] : [];
    const mixed = selected.length > 1 || Object.keys(input.task_ades ?? {}).length > 0;
    const assigned = (id: string, index: number) => input.task_ades?.[id] ?? (selected.length ? selected[index % selected.length] : null);
    const blocked_reasons: FlowPlan["blocked_reasons"] = [];
    const used = mixed ? tasks.map((task, index) => assigned(task.id, index)) : [input.ade_id];
    if (!used.length || used.some((id) => !id || !isBetaAde(id))) blocked_reasons.push({ kind: "permission_violation", message: "Desktop Beta runs Codex, Claude Code, Cursor Agent, OpenCode and Antigravity. Choose from those agents and build a new plan." });
    if (max_workers < 1 || max_workers > DESKTOP_BETA_MAX_WORKERS) blocked_reasons.push({ kind: "permission_violation", message: "Desktop Beta runs one to eight workers at once. Build a new plan within that limit." });
    return { draft_id: input.id, domain_id: input.domain_id ?? "pytxo", project_id: input.project_id, status: blocked_reasons.length ? "blocked" : "ready", tasks: tasks.map((task, index) => ({ ...task, verify: [...new Set([...(omitVerification ? [] : task.verify), ...(input.verification_commands ?? [])])], ...(mixed ? { ade_id: assigned(task.id, index) } : {}) })), max_workers, waves: max_workers === 1 ? tasks.map((task) => [task.id]) : [["desktop-flow", "orchestration-flow"], ["contract-tests"]], permission_profile: "orbit", isolation_mode: "copy_on_write", isolation_backend_intent: "projfs", execution_backend: "pty", ade: { requested: input.ade_id ?? null, available, installed: detected.filter((cli) => cli.installed).map((cli) => cli.id), command: requested?.default_cmd ?? null }, warnings: [{ code: "preview_fixture", message: "Browser preview uses a contract-valid Pytxo fixture; native preview reads the selected repository." }], blocked_reasons, estimated_tokens: null, estimated_cost_usd: null, previewed_at: new Date().toISOString() };
  }
  async experimentalClaudeRoutingAvailable() {
    return typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-claude-route-v1") === "1";
  }
  async experimentalHostedReviewAvailable() { return false; }
  async previewExperimentalClaudeHostedFlow(_input: FlowDraftInput, _facts: ReviewedDemandFacts): Promise<FlowPlan> {
    throw new Error("Hosted Shadow review requires the native experimental Desktop.");
  }
  async previewExperimentalClaudeFlow(input: FlowDraftInput, facts: ReviewedDemandFacts): Promise<FlowPlan> {
    if (!(await this.experimentalClaudeRoutingAvailable()) || input.ade_id !== null) {
      throw new Error("Claude routing experiment is unavailable in browser preview.");
    }
    return {
      draft_id: input.id,
      domain_id: input.domain_id ?? "pytxo",
      project_id: input.project_id,
      status: "ready",
      tasks: [{ id: "claude-proposal", agent: "Claude proposal route", prompt: input.mission_text, paths: ["src/api.ts"], dependencies: [], root: null, verify: input.verification_commands?.length ? input.verification_commands : ["npm run check"] }],
      waves: [["claude-proposal"]],
      max_workers: 1,
      permission_profile: "orbit",
      isolation_mode: "worktree",
      isolation_backend_intent: "worktree",
      execution_backend: "subprocess",
      ade: { requested: null, available: false, installed: [], command: null },
      warnings: [{ code: "claude_subscription_readiness_usage", message: "Dispatch runs live Claude subscription auth and model-readiness probes before the routed attempt. These may consume account quota even if no candidate is produced." }, { code: "preview_fixture", message: "Browser preview only; native routing requires a reviewed Store mission." }],
      blocked_reasons: [],
      estimated_tokens: null,
      estimated_cost_usd: null,
      previewed_at: new Date().toISOString(),
      routing: {
        authorization: {
          run_id: "run-preview",
          limits: {
            max_attempts: localStorage.getItem("pytxo-preview-claude-repair-v1") === "1"
              && ["documentation", "formatting", "rename", "local_transformation"].includes(facts.task_kind)
              && facts.context_complete && facts.cross_component_requirement === false ? 2 : 1,
          },
        },
        mission_digest: "browser-fixture-only",
      },
    };
  }
  async dispatchFlow() {
    if (localStorage.getItem("pytxo-preview-routed-stop-v1") === "1") {
      while (localStorage.getItem("pytxo-preview-routed-stop-requested-v1") !== "1") {
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
      throw new Error("Browser fixture: routed startup was stopped before a native run");
    }
    if (localStorage.getItem("pytxo-preview-flow-dispatch-recovery-v1") === "1") {
      localStorage.setItem("pytxo-preview-flow-history-v1", "routed-recovery");
      throw new Error("Browser fixture: routed startup requires recovery");
    }
    const domain = this.snapshot.domains[0];
    if (!this.snapshot.runs.some((run) => run.id === "run-preview")) {
      this.snapshot.runs.unshift({
        id: "run-preview",
        domain_id: domain?.domain_id ?? "pytxo",
        status: "running",
        repo_root: domain?.repo_root ?? "C:/dev/pytxo",
        started_at: new Date().toISOString(),
        estimated_cost_usd: 0,
        permission_profile: "orbit",
        isolation_mode: "copy_on_write",
        isolation_backend: "projfs",
        apply_status: "pending",
        applied_at: null,
        prepared_digest: null,
        prepared_at: null,
        last_apply_error: null,
        recovery_state: null,
        routing_revision: null,
      });
    }
    this.emitChange(domain?.domain_id ?? "pytxo", "run", "run-preview");
    setTimeout(() => {
      const run = this.snapshot.runs.find((item) => item.id === "run-preview");
      if (run) run.status = "completed";
      this.emitChange(domain?.domain_id ?? "pytxo", "run", "run-preview");
    }, 350);
    return "run-preview";
  }
  async stopRoutedFlow(): Promise<void> {
    if (localStorage.getItem("pytxo-preview-routed-stop-v1") !== "1") {
      throw new Error("Browser preview cannot stop a native routed run.");
    }
    localStorage.setItem("pytxo-preview-routed-stop-requested-v1", "1");
  }
  async saveReviewedFlow(plan: FlowPlan) { return plan; }
  async startVoice(device: string, language: string): Promise<VoiceSessionDto> {
    if (device === "Disconnected microphone") throw new Error("The input device was lost");
    if (this.activeVoiceSessions.size) throw new Error("another Voice session is already active");
    const session_id = crypto.randomUUID();
    this.voiceDevices.set(session_id, device);
    this.activeVoiceSessions.add(session_id);
    return { session_id, device, language, state: "recording", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null };
  }
  async pauseVoice(sessionId: string): Promise<VoiceSessionDto> { return { session_id: sessionId, device: "default", language: "en", state: "paused", elapsed_ms: 1200, buffered_samples: 16000, confidence: null, transcript_segments: [], error: null }; }
  async resumeVoice(sessionId: string): Promise<VoiceSessionDto> { return { session_id: sessionId, device: "default", language: "en", state: "recording", elapsed_ms: 1200, buffered_samples: 16000, confidence: null, transcript_segments: [], error: null }; }
  async finishVoice(sessionId: string): Promise<VoiceSessionDto> {
    // Keep the browser-preview cancellation affordance operable long enough
    // for a human or E2E pointer action; native transcription owns its timing.
    await new Promise((resolve) => setTimeout(resolve, 650));
    const device = this.voiceDevices.get(sessionId) ?? "default";
    this.activeVoiceSessions.delete(sessionId);
    if (this.cancelledVoiceSessions.has(sessionId)) return { session_id: sessionId, device, language: "en", state: "cancelled", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null };
    const uncertain = device === "Studio microphone";
    return { session_id: sessionId, device, language: "en", state: "ready", elapsed_ms: 3200, buffered_samples: 0, confidence: uncertain ? 0.52 : 0.94, transcript_segments: [{ text: uncertain ? "Update the desktop approval flour" : "Create a production-ready Flow mission", confidence: uncertain ? 0.52 : 0.94, uncertain }], error: null };
  }
  async cancelVoice(sessionId: string): Promise<VoiceSessionDto> { this.cancelledVoiceSessions.add(sessionId); this.activeVoiceSessions.delete(sessionId); return { session_id: sessionId, device: this.voiceDevices.get(sessionId) ?? "default", language: "en", state: "cancelled", elapsed_ms: 0, buffered_samples: 0, confidence: null, transcript_segments: [], error: null }; }
  async listVoiceDevices() { return ["default", "Studio microphone", "Disconnected microphone"]; }
  async onVoiceProgress() { return () => {}; }
  async voiceModelStatus() { return null; }
  async installVoiceModel() { return "~/.pytxo/models/voice/base.en.bin"; }
  async openWorkspace() { return "C:/dev/new-workspace"; }
  async createExampleWorkspace() {
    if (typeof localStorage !== "undefined" && localStorage.getItem("pytxo-preview-example-error-v1") === "missing-git") {
      throw new Error("Git was not found. Install Git, restart Pytxo Desktop, then try again. You can skip this step for now.");
    }
    const path = "C:/Users/demo/Documents/Pytxo Examples/approval-risk-demo";
    if (!this.snapshot.domains.some((domain) => domain.repo_root === path)) {
      this.snapshot.domains.push({
        domain_id: "approval-risk-demo",
        repo_root: path,
        db_path: "~/.pytxo/domains/approval-risk-demo.db",
        project_id: null,
        status: "healthy",
        updated_at: new Date().toISOString(),
        active_runs: 0,
        latest_run_status: null,
        latest_started_at: null,
        hitl_pending: 0,
        is_available: true,
        is_temporary: false,
      });
    }
    return path;
  }
  async voiceAvailable() { return true; }
  async flowHistory(): Promise<FlowDraftRecord[]> {
    const fixture = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-flow-history-v1");
    if (!fixture) return [];
    if (fixture.startsWith("advisor-")) {
      return [{ id: "advisor-review", title: "Reviewed routing task", mission_text: "Browser fixture for a reviewed task", source: "text", domain_id: "signal-lab", project_id: null, status: "ready", plan_json: JSON.stringify({ status: "ready", routing: { authorization: { run_id: "browser-fixture-run" } } }), dispatched_run_id: null, created_at: "2026-09-06T12:00:00Z", updated_at: "2026-09-06T12:00:00Z" }];
    }
    if (fixture === "long-mission" || fixture === "long-worker-request") {
      const mission = "Update the parser while preserving its public API; add regression tests for Windows paths and mixed-case input; document the final behavior and exact examples. ".repeat(6);
      return [{ id: "draft-density", title: mission.slice(0, 70), mission_text: mission, source: "text", domain_id: "pytxo", project_id: null, status: "completed", plan_json: JSON.stringify({ tasks: [{ id: "plan", prompt: "Inspect the existing review contract" }, { id: "ui", prompt: fixture === "long-worker-request" ? mission : "Clarify the candidate review experience" }, { id: "tests", prompt: "Check the reviewed candidate safeguards" }] }), dispatched_run_id: "run-8f2c", created_at: "2026-09-06T12:00:00Z", updated_at: "2026-09-06T12:00:00Z" }];
    }
    const domainId = fixture === "long-path" ? `C:/workspaces/${"a-very-long-project-folder/".repeat(8)}repository` : fixture === "missing" ? "unavailable-workspace" : "signal-lab";
    return [{ id: "draft-reuse", title: "Fix the parser regression", mission_text: "Fix src/parser.rs and add a regression test", source: "text", domain_id: domainId, project_id: null, status: fixture === "routed-recovery" ? "recovery_required" : fixture === "routed-failed" ? "failed" : fixture === "routed-dispatching" ? "dispatching" : "completed", plan_json: JSON.stringify({ ade: { requested: fixture === "unavailable-cli" ? "claude" : "codex" }, ...(fixture === "routed-dispatching" && localStorage.getItem("pytxo-preview-routed-stop-v1") === "1" ? { routing: { authorization: { run_id: "run-71ad" } } } : {}) }), dispatched_run_id: "run-71ad", created_at: "2026-09-06T12:00:00Z", updated_at: "2026-09-06T12:00:00Z" }];
  }
  async deleteFlowDraft() {}
  async readAgentEvents(runId: string, agentId: string, domainId: string, after: number, limit: number): Promise<import("./types").EventDto[]> {
    const agent = this.snapshot.agents.find(a => a.id === agentId && a.run_id === runId && a.domain_id === domainId);
    if (!agent) throw new Error("Agent not present in this preview scope.");
    if (localStorage.getItem("pytxo-preview-observe-polls-v1") === "1") {
      const reads = Number(localStorage.getItem("pytxo-preview-agent-event-reads-v1") ?? "0");
      localStorage.setItem("pytxo-preview-agent-event-reads-v1", String(reads + 1));
    }
    const fleetTask = localStorage.getItem("pytxo-preview-fleet-v1") === "1" ? fleetTasks.find(task => agentId.endsWith(`:fleet-${task.task_id}`)) : undefined;
    if (fleetTask) {
      const start = Date.now() - 140_000 + fleetTask.wave * 40_000;
      return fleetTask.lines.map((payload, index) => ({ id: index + 1, agent_id: agentId, kind: index === 0 && payload.startsWith("$ ") ? "verify" : "stdout", payload: index === 0 && payload.startsWith("$ ") ? payload.slice(2) : `${payload}
`, ts: new Date(start + index * 9_000).toISOString() }))
        .concat(fleetTask.status === "completed" ? [{ id: fleetTask.lines.length + 1, agent_id: agentId, kind: "verify-ok", payload: "", ts: new Date(start + 58_000).toISOString() }] : [])
        .filter(e => e.id > after).slice(0, limit);
    }
    if (localStorage.getItem("pytxo-preview-output-events-v1") === "600") {
      return Array.from({ length: 600 }, (_, index) => ({ id: index + 1, agent_id: agentId,
        kind: index % 19 === 0 ? "tool" : "stdout",
        payload: index % 19 === 0 ? `Browser fixture: tool record ${index}.` : `Browser fixture: worker output ${index}; read only and unverified.`,
        ts: "2026-09-20T08:00:00Z",
      })).filter(event => event.id > after).slice(0, limit);
    }
    if (localStorage.getItem("pytxo-preview-layout-fixture-v1") === "long") {
      return Array.from({ length: 80 }, (_, index) => ({ id: index + 1, agent_id: agentId,
        kind: index === 0 ? "agent-start" : index === 79 ? "verify-ok" : "stdout",
        payload: index === 0 ? "Browser fixture: runner started the selected task." : index === 79 ? "Browser fixture: verification command completed." : `Browser fixture: worker prose ${index}; this is not control evidence.`,
        ts: "2026-09-13T08:00:00Z",
      })).filter(e => e.id > after).slice(0, limit);
    }
    return ["\x1b[", "32mBrowser fixture: recorded output example. Native Desktop reads actual stored events.", "\x1b[0m"].map((payload, index) => ({ id: index + 1, agent_id: agentId, kind: "stdout", payload, ts: "2026-09-12T00:00:00Z" })).filter(e => e.id > after).slice(0, limit);
  }
  async agentFailureHint(runId: string, agentId: string, domainId: string): Promise<string | null> {
    const agent = this.snapshot.agents.find(a => a.id === agentId && a.run_id === runId && a.domain_id === domainId);
    if (!agent) throw new Error("Agent not present in this preview scope.");
    return agent.status === "failed" ? "Browser fixture: the agent CLI reported an error before finishing." : null;
  }
  async listAgents(runId: string) {
    const verification = localStorage.getItem("pytxo-preview-agent-verification-v1");
    if (runId === "run-71ad" && verification === "missing") {
      return [];
    }
    const agents = structuredClone(
      [...this.snapshot.agents, ...previewCompletedAgents].filter(
        (agent) => agent.run_id === runId,
      ),
    );
    if (runId === "run-71ad" && verification === "failed" && agents[0]) {
      agents[0].exit_code = 1;
      agents[0].status = "failed";
    }
    return agents;
  }
  async runReview(runId: string): Promise<RunReviewDto> {
    if (localStorage.getItem("pytxo-preview-delayed-plan-v1") === "1") {
      await new Promise(resolve => setTimeout(resolve, 1200));
    }
    if (localStorage.getItem("pytxo-preview-observe-polls-v1") === "1") {
      const reads = Number(localStorage.getItem("pytxo-preview-review-reads-v1") ?? "0");
      localStorage.setItem("pytxo-preview-review-reads-v1", String(reads + 1));
    }
    if (localStorage.getItem("pytxo-preview-history-evidence-error-v1") === "1" && runId === "run-71ad") throw new Error("Browser fixture: stored evidence unavailable");
    const delayedOtherRun = runId === "run-other";
    if (delayedOtherRun) await new Promise((resolve) => setTimeout(resolve, 1500));
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    const isSignalRun = runId === "run-71ad";
    const denseTopology = localStorage.getItem("pytxo-preview-topology-v1") === "24";
    const densePlan: RunReviewDto["plan"] = {
      waves: Array.from({ length: 6 }, (_, wave) => Array.from({ length: 4 }, (_, index) => {
        const taskId = wave === 0 && index === 0 ? "plan" : wave === 1 && index === 0 ? "ui" : wave === 2 && index === 0 ? "tests" : `dense-${wave}-${index}`;
        const previousId = wave - 1 === 0 && index === 0 ? "plan" : wave - 1 === 1 && index === 0 ? "ui" : wave - 1 === 2 && index === 0 ? "tests" : `dense-${wave - 1}-${index}`;
        const dependency = wave === 0 ? [] : [previousId];
        return { task_id: taskId, agent: "codex", paths: [`src/${taskId}.ts`], depends_on: dependency, wave, root: null, verify: ["npm test"] };
      })),
      warnings: [],
    };
    const fleetPlan: RunReviewDto["plan"] = {
      waves: [0, 1, 2].map(wave => fleetTasks.filter(task => task.wave === wave).map(task => ({ task_id: task.task_id, agent: task.cli, paths: task.paths, depends_on: task.depends_on, wave, root: null, verify: ["node --test"] }))),
      warnings: [],
    };
    const fleet = localStorage.getItem("pytxo-preview-fleet-v1") === "1" && runId === this.snapshot.runs[0]?.id;
    const plan = fleet ? fleetPlan : denseTopology ? densePlan : isSignalRun
      ? {
          waves: [
            [{
              task_id: "signal-core",
              agent: "codex",
              paths: ["crates/pytxo-signal/src/lib.rs"],
              depends_on: [],
              wave: 0,
              root: null,
              verify: ["cargo test -p pytxo-signal"],
            }],
            [{
              task_id: "contract-tests",
              agent: "codex",
              paths: ["crates/pytxo-signal/tests/skeleton.rs"],
              depends_on: ["signal-core"],
              wave: 1,
              root: null,
              verify: ["cargo test -p pytxo-signal"],
            }],
          ],
          warnings: [],
        }
      : {
          waves: [
            [{
              task_id: "plan",
              agent: "architect",
              paths: ["docs/02-areas/orchestration/signal-core.md"],
              depends_on: [],
              wave: 0,
              root: null,
              verify: [],
            }],
            [{
              task_id: "ui",
              agent: "desktop",
              paths: ["apps/desktop/src/components/desktop2/FocusScreen.svelte"],
              depends_on: ["plan"],
              wave: 1,
              root: null,
              verify: ["npm run check"],
            }],
            [{
              task_id: "tests",
              agent: "verification",
              paths: ["crates/pytxo-orchestrate/tests/run_apply.rs"],
              depends_on: ["ui"],
              wave: 2,
              root: null,
              verify: ["cargo test -p pytxo-orchestrate"],
            }],
          ],
          warnings: ["Browser preview reflects a live run fixture; Apply remains disabled until every agent exits cleanly."],
        };
    const preparedManifest: PreparedRunManifest = {
      version: 2,
      run_id: runId,
      base_revision: isSignalRun ? "71ad8f2c4d90b6c6" : "8f2cc9814fc10e31",
      prepared_at: "2026-08-01T02:30:00Z",
      package_digest: (isSignalRun ? "pkg-71ad-immutable" : "pkg-8f2c-immutable")
        + (localStorage.getItem(`pytxo-preview-review-revision:${runId}`) ?? ""),
      summary: { added: 2, modified: 1, deleted: 1, bytes: 1847 },
      files: [
        {
          path: "crates/pytxo-signal/src/lib.rs",
          kind: "modify",
          before_sha256: "2e0f6b77",
          after_sha256: "e51c34aa",
          byte_count: 1120,
          task_id: "signal-core",
          agent_id: "run-71ad:agent-0",
          blob_digest: "e51c34aa",
          before_mode: null,
          after_mode: null,
          before_byte_count: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/lib.rs"].before!).length,
          after_byte_count: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/lib.rs"].after!).length,
          before_is_binary: false,
          after_is_binary: false,
          before_chunks: [{ offset: 0, length: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/lib.rs"].before!).length, sha256: "preview-lib-before-chunk" }],
          after_chunks: [{ offset: 0, length: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/lib.rs"].after!).length, sha256: "preview-lib-after-chunk" }],
        },
        {
          path: "crates/pytxo-signal/tests/skeleton.rs",
          kind: "add",
          before_sha256: null,
          after_sha256: "1a9d80c3",
          byte_count: 612,
          task_id: "contract-tests",
          agent_id: "run-71ad:agent-1",
          blob_digest: "1a9d80c3",
          before_mode: null,
          after_mode: null,
          before_byte_count: 0,
          after_byte_count: previewBytes(reviewContentFixture()["crates/pytxo-signal/tests/skeleton.rs"].after!).length,
          before_is_binary: null,
          after_is_binary: false,
          before_chunks: [],
          after_chunks: [{ offset: 0, length: previewBytes(reviewContentFixture()["crates/pytxo-signal/tests/skeleton.rs"].after!).length, sha256: "preview-test-after-chunk" }],
        },
        {
          path: "crates/pytxo-signal/src/legacy.rs",
          kind: "delete",
          before_sha256: "f1e2d3c4",
          after_sha256: null,
          byte_count: 110,
          task_id: "signal-core",
          agent_id: "run-71ad:agent-0",
          blob_digest: null,
          before_mode: null,
          after_mode: null,
          before_byte_count: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/legacy.rs"].before!).length,
          after_byte_count: 0,
          before_is_binary: false,
          after_is_binary: null,
          before_chunks: [{ offset: 0, length: previewBytes(reviewContentFixture()["crates/pytxo-signal/src/legacy.rs"].before!).length, sha256: "preview-legacy-before-chunk" }],
          after_chunks: [],
        },
        {
          path: "assets/signal-mark.bin",
          kind: "add",
          before_sha256: null,
          after_sha256: "00ff504e47",
          byte_count: previewSignalMark.length,
          task_id: "contract-tests",
          agent_id: "run-71ad:agent-1",
          blob_digest: "00ff504e47",
          before_mode: null,
          after_mode: null,
          before_byte_count: 0,
          after_byte_count: previewSignalMark.length,
          before_is_binary: null,
          after_is_binary: true,
          before_chunks: [],
          after_chunks: [{ offset: 0, length: previewSignalMark.length, sha256: "preview-signal-mark-chunk" }],
        },
      ],
    };
    const appliedManifest: RunApplyManifest | null = run.apply_status === "applied"
      ? {
          transaction_id: "apply-71ad-20260731",
          changes: [
            {
              path: "crates/pytxo-signal/src/lib.rs",
              kind: "modify",
              source_agent_id: "run-71ad:agent-0",
              source_task_id: "signal-core",
              base_digest: "2e0f6b77",
              result_digest: "e51c34aa",
            },
            {
              path: "crates/pytxo-signal/tests/skeleton.rs",
              kind: "add",
              source_agent_id: "run-71ad:agent-1",
              source_task_id: "contract-tests",
              base_digest: null,
              result_digest: "1a9d80c3",
            },
            {
              path: "crates/pytxo-signal/src/legacy.rs",
              kind: "delete",
              source_agent_id: "run-71ad:agent-0",
              source_task_id: "signal-core",
              base_digest: "f1e2d3c4",
              result_digest: null,
            },
          ],
        }
      : null;
    const requested =
      typeof localStorage === "undefined"
        ? null
        : localStorage.getItem("pytxo-preview-review-state-v1");
    const candidateCheck = typeof localStorage === "undefined" ? null : localStorage.getItem("pytxo-preview-candidate-check-v1");
    if (candidateCheck) {
      preparedManifest.version = 3;
      preparedManifest.candidate_verification = {
        version: 1,
        verified_at: "2026-09-06T02:30:00Z",
        base_inventory: [],
        candidate_inventory: [],
        exclusions: [".git", "node_modules"],
        checks: candidateCheck === "empty" ? [] : [{
          task_id: "ui",
          command: "npm run check",
          effective_profile: "orbit",
          passed: candidateCheck === "passed",
          enforcement: {},
        }],
      };
    }
    const recovered = requested === "recovered" || this.recoveredRuns.has(runId)
      || runId === "run-8f2c" && localStorage.getItem("pytxo-preview-legacy-retry-v1") === "1" && localStorage.getItem("pytxo-preview-newer-apply-error-v1") !== "1";
    const lastError: RunApplyError | null =
      recovered
        ? {
            at: "2026-08-01T02:20:00Z",
            code: "apply_failed",
            message: "The previous Apply failed and was rolled back.",
            attempt_id: "attempt-preview-1",
            rollback_confirmed: true,
          }
        : run.apply_status === "stale"
          ? {
              at: "2026-08-01T02:25:00Z",
              code: "source_drift",
              message: "An affected checkout path changed after review.",
              attempt_id: null,
              rollback_confirmed: false,
            }
          : run.apply_status === "recovery_required"
            ? {
                at: "2026-08-01T02:26:00Z",
                code: "apply_recovery_required",
                message: "Apply is blocked until recovery is reconciled.",
                attempt_id: "attempt-preview-2",
                rollback_confirmed: false,
              }
            : null;
    const applyAttempts = [
      ...(run.apply_status === "applied"
        ? [{
            attempt_id: "apply-71ad-20260731",
            created_at: run.applied_at,
            phase: "committed",
            outcome: "committed" as const,
            error_code: null,
            error_message: null,
            rollback_confirmed: false,
          }]
        : []),
      ...(recovered
        ? [
            {
              attempt_id: "attempt-preview-1",
              created_at: "2026-08-01T02:20:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "The previous Apply failed and was rolled back.",
              rollback_confirmed: true,
            },
            {
              attempt_id: "attempt-preview-0",
              created_at: "2026-08-01T02:10:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "A prior filesystem write failed and was rolled back.",
              rollback_confirmed: true,
            },
          ]
        : []),
      ...(stateMatrixRequested()
        ? [
            {
              attempt_id: "attempt-matrix-committed",
              created_at: "2026-08-01T03:00:00Z",
              phase: "committed",
              outcome: "committed" as const,
              error_code: null,
              error_message: null,
              rollback_confirmed: false,
            },
            {
              attempt_id: "attempt-matrix-rolled-back",
              created_at: "2026-08-01T03:05:00Z",
              phase: "rolled_back",
              outcome: "rolled_back" as const,
              error_code: "apply_failed",
              error_message: "A write failed and the attempt was rolled back.",
              rollback_confirmed: true,
            },
            {
              attempt_id: "attempt-matrix-recovery",
              created_at: "2026-08-01T03:10:00Z",
              phase: "recovery_required",
              outcome: "recovery_required" as const,
              error_code: "apply_recovery_required",
              error_message: "Apply is blocked until recovery is reconciled.",
              rollback_confirmed: false,
            },
            {
              attempt_id: "attempt-matrix-interrupted",
              created_at: "2026-08-01T03:15:00Z",
              phase: "interrupted",
              outcome: "interrupted" as const,
              error_code: null,
              error_message: "The process stopped before reporting an outcome.",
              rollback_confirmed: false,
            },
          ]
        : []),
    ];
    const receipt = stateMatrixRequested() || delayedOtherRun ? previewMatrixReceipt : previewReceipt;
    return {
      run_id: runId,
      base_revision: isSignalRun ? "71ad8f2c4d90b6c6" : "8f2cc9814fc10e31",
      apply_status: run.apply_status ?? "pending",
      applied_at: run.applied_at,
      plan,
      enforcement: {
        run: structuredClone(receipt),
        agents: Object.fromEntries(
          (await this.listAgents(runId))
            .map((agent) => [agent.id.startsWith(`${runId}:`) ? agent.id.slice(runId.length + 1) : agent.id, structuredClone(receipt)]),
        ),
      },
      apply_manifest: appliedManifest,
      prepared_manifest: run.apply_status === "review_failed" ? null : preparedManifest,
      prepared_digest: run.apply_status === "review_failed" ? null : preparedManifest.package_digest,
      prepared_at: run.apply_status === "review_failed" ? null : preparedManifest.prepared_at,
      last_apply_error: run.last_apply_error ?? lastError,
      recovery_state: recovered
        ? "rolled_back"
        : run.apply_status === "stale"
          ? "source_drift"
          : run.apply_status === "recovery_required"
            ? "unprovable"
            : null,
      apply_attempts: applyAttempts,
    };
  }
  async runReviewContent(
    runId: string,
    path: string,
    side: "before" | "after",
    offset: number,
    limit: number,
  ): Promise<PreparedContentChunkDto> {
    const value = reviewContentFixture()[path]?.[side];
    if (value == null) throw new Error(`${side} content does not exist for ${path}`);
    const review = await this.runReview(runId);
    const file = review.prepared_manifest?.files.find((candidate) => candidate.path === path);
    const digest = side === "before" ? file?.before_sha256 : file?.after_sha256;
    if (!review.prepared_manifest || !digest) throw new Error(`${side} content identity is unavailable for ${path}`);
    const requestKey = "pytxo-preview-review-content-requests-v1";
    const requests = JSON.parse(localStorage.getItem(requestKey) ?? "[]") as Array<{ path: string; side: "before" | "after" }>;
    requests.push({ path, side });
    localStorage.setItem(requestKey, JSON.stringify(requests));
    const activeKey = "pytxo-preview-review-content-active-v1";
    const maxActiveKey = "pytxo-preview-review-content-max-active-v1";
    const active = Number(localStorage.getItem(activeKey) ?? "0") + 1;
    localStorage.setItem(activeKey, String(active));
    localStorage.setItem(
      maxActiveKey,
      String(Math.max(active, Number(localStorage.getItem(maxActiveKey) ?? "0"))),
    );
    try {
      const delay = Number(localStorage.getItem("pytxo-preview-review-content-delay-v1") ?? "0");
      if (delay > 0) await new Promise((resolve) => setTimeout(resolve, delay));
      const bytes = previewBytes(value);
      const chunk = bytes.slice(offset, Math.min(bytes.length, offset + Math.min(limit, 64 * 1024)));
      const nextOffset = offset + chunk.length;
      return {
        run_id: runId,
        package_digest: review.prepared_manifest.package_digest,
        path,
        side,
        digest,
        byte_count: bytes.length,
        binary: value instanceof Uint8Array,
        offset,
        length: chunk.length,
        next_offset: nextOffset,
        complete: nextOffset === bytes.length,
        data_base64: bytesToBase64(chunk),
      };
    } finally {
      const remaining = Math.max(0, Number(localStorage.getItem(activeKey) ?? "1") - 1);
      localStorage.setItem(activeKey, String(remaining));
    }
  }
  async refreshRunReview(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "ready";
    localStorage.setItem(`pytxo-preview-review-revision:${runId}`, `-refresh-${Date.now()}`);
    localStorage.setItem("pytxo-preview-candidate-check-v1", "passed");
    this.emitChange("signal-lab", "contract", runId);
    return (await this.runReview(runId)).prepared_manifest!;
  }
  async discardRunReview(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "discarded";
    this.emitChange("signal-lab", "contract", runId);
  }
  async reconcileRunRecovery(runId: string) {
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    run.apply_status = "ready";
    this.recoveredRuns.add(runId);
    this.emitChange("signal-lab", "contract", runId);
    return { outcome: "rolled_back", attempt_id: "attempt-preview-2" };
  }
  async domainChanges(_domainId: string, cursor: number, limit = 200) {
    if (cursor > this.sequence) {
      return {
        changes: [],
        next_cursor: this.sequence,
        has_more: false,
        cursor_gap: true,
      };
    }
    const page = this.changes.filter((change) => change.sequence > cursor).slice(0, limit);
    return {
      changes: structuredClone(page),
      next_cursor: page.at(-1)?.sequence ?? cursor,
      has_more: this.changes.some((change) => change.sequence > (page.at(-1)?.sequence ?? cursor)),
      cursor_gap: false,
    };
  }
  async previewFlowAdvisorPacket(draftId: string): Promise<RoutedAdvisorPacketPreview> {
    if (draftId !== "advisor-review" || localStorage.getItem("pytxo-preview-flow-history-v1") !== "advisor-shadow") {
      throw new Error("Browser fixture: no current ready Shadow packet is available");
    }
    const state = { schema_version: 1, goal: "Classify reviewed repository task using coarse facts", features: ["task_kind_local_transformation", "reviewed_checks", "single_component_claimed", "context_claimed_complete", "role_unrestricted", "no_required_egress"], everyday_role: "everyday execution role", strong_role: "strong execution role" };
    const body = new TextEncoder().encode(JSON.stringify({ model: "browser-fixture-no-send", state, questions: { browser_fixture: { type: "choice" } } }));
    const hexDigest = async (bytes: Uint8Array<ArrayBuffer>) => [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map((value) => value.toString(16).padStart(2, "0")).join("");
    return { domain_id: "signal-lab", run_id: "browser-fixture-run", task_id: "browser-fixture-task", reviewed_consent_revision: 1, recipient_identity: "pytxo-local-advisor-fixture/no-network/v1", packet_digest: await hexDigest(new TextEncoder().encode(JSON.stringify(state))), request_digest: await hexDigest(body), request_body: [...body] };
  }

  async previewProposedHostedPacket(draftId: string): Promise<ProposedHostedAdvisorPacketPreview> {
    const local = await this.previewFlowAdvisorPacket(draftId);
    const request = JSON.parse(new TextDecoder().decode(Uint8Array.from(local.request_body)));
    const packet_body = [...new TextEncoder().encode(JSON.stringify(request.state))];
    const scope = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode("browser-fixture-proposed-hosted-scope-v1")));
    return { domain_id: local.domain_id, run_id: local.run_id, task_id: local.task_id, source_review_recipient_identity: local.recipient_identity, recipient_identity: "pytxo-hosted-routing/typesafe-systemone/v1", scope_digest: [...scope].map((value) => value.toString(16).padStart(2, "0")).join(""), packet_digest: local.packet_digest, wire_schema_version: 1, decision_kind: "initial_demand", question_set_version: "execution_demand_v2", packet_body };
  }

  async previewReviewedHostedPacket(_draftId: string): Promise<ReviewedHostedAdvisorPacketPreview> {
    throw new Error("Hosted Shadow review requires the native experimental Desktop.");
  }
  async hostedConsentStatus(_domainId: string): Promise<RoutedAdvisorConsentStatus> {
    throw new Error("Hosted consent requires the native experimental Desktop.");
  }
  async enableHostedConsent(_draftId: string, _domainId: string, _requestDigest: string, _scopeDigest: string, _expectedRevision: number): Promise<RoutedAdvisorConsentStatus> {
    throw new Error("Hosted consent requires the native experimental Desktop.");
  }
  async revokeHostedConsent(_domainId: string, _expectedRevision: number): Promise<RoutedAdvisorConsentStatus> {
    throw new Error("Hosted consent requires the native experimental Desktop.");
  }
  async hostedGrantStatus(_domainId: string): Promise<RoutingHostedGrantStatus | null> { return null; }
  async hostedGrants(): Promise<RoutingHostedGrantStatus[]> { return []; }
  async enableHostedGrant(_domainId: string): Promise<RoutingHostedGrantStatus> {
    throw new Error("Hosted grants require the native experimental Desktop.");
  }
  async revokeHostedGrant(_domainId: string, _expectedRevision: number): Promise<RoutingHostedGrantStatus> {
    throw new Error("Hosted grants require the native experimental Desktop.");
  }

  async flowAdvisorConsentDomains(): Promise<string[]> {
    return localStorage.getItem(this.shadowConsentStorageKey) ? [this.shadowConsent.domain_id] : [];
  }

  async flowAdvisorConsent(domainId: string): Promise<RoutedAdvisorConsentStatus> {
    if (domainId !== this.shadowConsent.domain_id) throw new Error("Browser fixture: wrong routing workspace");
    if (localStorage.getItem("pytxo-preview-shadow-consent-error-v1") === "1") throw new Error("Browser fixture: saved routing grant unavailable");
    const saved = localStorage.getItem(this.shadowConsentStorageKey);
    if (saved) {
      try {
        const parsed: unknown = JSON.parse(saved);
        if (parsed && typeof parsed === "object" && "domain_id" in parsed && parsed.domain_id === domainId && "revision" in parsed && typeof parsed.revision === "number" && Number.isSafeInteger(parsed.revision) && "enabled" in parsed && typeof parsed.enabled === "boolean" && "current_scope" in parsed && typeof parsed.current_scope === "boolean" && "recipient_identity" in parsed && parsed.recipient_identity === this.shadowConsent.recipient_identity && "updated_at_ms" in parsed && typeof parsed.updated_at_ms === "number") {
          this.shadowConsent = parsed as RoutedAdvisorConsentStatus;
        }
      } catch { /* A corrupt browser fixture is ignored. Native Store is authoritative. */ }
    }
    if (this.shadowConsent.revision === 0 && localStorage.getItem("pytxo-preview-shadow-consent-v1") === "other-scope") {
      this.shadowConsent = { ...this.shadowConsent, revision: 1, enabled: true, current_scope: false, updated_at_ms: Date.now() };
    }
    return structuredClone(this.shadowConsent);
  }

  async enableFlowAdvisorConsent(draftId: string, domainId: string, requestDigest: string, recipientIdentity: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus> {
    const packet = await this.previewFlowAdvisorPacket(draftId);
    await this.flowAdvisorConsent(domainId);
    if (packet.domain_id !== domainId || packet.request_digest !== requestDigest || packet.recipient_identity !== recipientIdentity || this.shadowConsent.revision !== expectedRevision || packet.reviewed_consent_revision !== expectedRevision + 1) {
      throw new Error("Browser fixture: inspected Shadow packet or consent revision changed");
    }
    this.shadowConsent = { ...this.shadowConsent, revision: expectedRevision + 1, enabled: true, current_scope: true, updated_at_ms: Date.now() };
    localStorage.setItem(this.shadowConsentStorageKey, JSON.stringify(this.shadowConsent));
    return structuredClone(this.shadowConsent);
  }

  async revokeFlowAdvisorConsent(domainId: string, expectedRevision: number): Promise<RoutedAdvisorConsentStatus> {
    await this.flowAdvisorConsent(domainId);
    if (domainId !== this.shadowConsent.domain_id || this.shadowConsent.revision !== expectedRevision) throw new Error("Browser fixture: stale routing workspace consent");
    this.shadowConsent = { ...this.shadowConsent, revision: expectedRevision + 1, enabled: false, current_scope: false, updated_at_ms: Date.now() };
    localStorage.setItem(this.shadowConsentStorageKey, JSON.stringify(this.shadowConsent));
    return structuredClone(this.shadowConsent);
  }

  async routingRunSummary(runId: string, domainId: string): Promise<RoutingDisplaySummary | null> {
    if (localStorage.getItem("pytxo-preview-routing-summary-v1") !== "1" || runId !== "run-8f2c") return null;
    if (localStorage.getItem("pytxo-preview-routing-error-v1") === "1") throw new Error("Browser fixture: routing read unavailable");
    if (domainId === "pytxo" && localStorage.getItem("pytxo-preview-routing-delay-v1") === "1") {
      await new Promise(resolve => setTimeout(resolve, 900));
    }
    if (domainId === "signal-lab" && localStorage.getItem("pytxo-preview-routing-same-id-v1") === "1") {
      return { ...structuredClone(previewRoutingSummary), domain_id: domainId, tasks: [{ task_id: "other-domain-task", state: "ready", dependency_task_ids: [], current_attempt_id: null, winning_attempt_id: null, last_decision: null, pre_admission: null, attempts: [] }] };
    }
    if (domainId !== "pytxo") return null;
    const record = structuredClone(previewRoutingSummary);
    if (localStorage.getItem("pytxo-preview-routed-retry-workers-v1") === "1") {
      record.tasks[1].attempts[1].state = "running";
    }
    if (localStorage.getItem("pytxo-preview-routing-cancelled-v1") === "1") {
      record.cancelled = true;
      record.tasks[1].state = "cancelled";
      record.tasks[2].state = "cancelled";
    }
    if (localStorage.getItem("pytxo-preview-routing-later-block-v1") === "1") {
      record.tasks[2].state = "waiting_input";
      record.tasks[2].last_decision = {
        selection: { kind: "blocked", code: "budget_exhausted" },
        reason: "blocked",
        advice_status: "not_used",
      };
    }
    if (localStorage.getItem("pytxo-preview-routing-stale-revision-v1") === "1") record.routing_revision = "2";
    return record;
  }
  async catalogFingerprint() {
    const fixture = localStorage.getItem("pytxo-preview-catalog-fingerprint-v1");
    if (fixture) return fixture;
    return this.snapshot.domains
      .map((domain) => `${domain.domain_id}:${domain.updated_at}`)
      .sort()
      .join("|");
  }
  async onDomainChanged(callback: (event: DesktopChangedEvent) => void) {
    this.domainListeners.add(callback);
    // Two browser clients share the fixture's durable package revision. Native
    // clients receive the equivalent domain-change notification through IPC.
    const onStorage = (event: StorageEvent) => {
      const prefix = "pytxo-preview-review-revision:";
      if (!event.key?.startsWith(prefix)) return;
      const runId = event.key.slice(prefix.length);
      const run = this.snapshot.runs.find((item) => item.id === runId);
      if (run) callback({ domain_id: run.domain_id, entity_kind: "contract", entity_id: runId });
    };
    window.addEventListener("storage", onStorage);
    return () => {
      this.domainListeners.delete(callback);
      window.removeEventListener("storage", onStorage);
    };
  }
  async structuralGraph() {
    return {
      version: 3,
      nodes: [
        { id: "signal-core", label: "SignalCore", edited: true, root_id: null },
        { id: "parser", label: "Parser", edited: true, root_id: null },
        { id: "skeleton", label: "Skeleton", edited: false, root_id: null },
        { id: "tests", label: "contract tests", edited: true, root_id: null },
      ],
      edges: [
        { from: "signal-core", to: "parser" },
        { from: "parser", to: "skeleton" },
        { from: "tests", to: "signal-core" },
      ],
    };
  }
  async workspaceStructuralGraph() {
    return this.structuralGraph();
  }
  async agentArbitrage(runId: string) {
    return (await this.listAgents(runId))
      .map((agent, index) => ({
        agent_id: agent.id,
        saved_tokens: 3200 - (index * 450),
        edited_paths: index + 1,
        fallback_paths: 0,
      }));
  }
  async applyRunChanges(runId: string, _domainId: string | null, expectedPackageDigest: string): Promise<RunApplyManifest> {
    const current = await this.runReview(runId);
    if (!expectedPackageDigest || expectedPackageDigest !== current.prepared_digest) {
      throw new Error("[stale_review] The reviewed candidate changed. Reload and review the current candidate before Apply.");
    }
    const run = this.snapshot.runs.find((item) => item.id === runId);
    if (!run) throw new Error(`Run not found: ${runId}`);
    const review = await this.runReview(runId);
    const manifest = review.apply_manifest && !("error" in review.apply_manifest)
      ? review.apply_manifest
      : {
          transaction_id: `apply-${runId}`,
          changes: [],
        };
    run.apply_status = "applying";
    const count = Number(localStorage.getItem("pytxo-preview-apply-count-v1") ?? "0") + 1;
    localStorage.setItem("pytxo-preview-apply-count-v1", String(count));
    this.emitChange("signal-lab", "contract", runId);
    await new Promise((resolve) => setTimeout(resolve, 120));
    if (localStorage.getItem("pytxo-preview-apply-outcome-v1") === "stale") {
      run.apply_status = "stale";
      run.last_apply_error = {
        at: new Date().toISOString(),
        code: "source_drift",
        message: "An affected checkout path changed after review.",
        attempt_id: null,
        rollback_confirmed: false,
      };
      run.recovery_state = "source_drift";
      this.emitChange("signal-lab", "contract", runId);
      throw new Error("An affected checkout path changed after review.");
    }
    run.apply_status = "applied";
    run.applied_at = new Date().toISOString();
    run.last_apply_error = null;
    run.recovery_state = null;
    this.emitChange("signal-lab", "contract", runId);
    return manifest;
  }
}
