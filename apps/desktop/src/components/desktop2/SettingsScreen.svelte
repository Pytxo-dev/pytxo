<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import {
    IconBell,
    IconFolder,
    IconKey,
    IconKeyboard,
    IconMicrophone,
    IconPalette,
    IconSearch,
    IconSettings,
    IconShieldLock,
    IconUserCircle,
  } from "@tabler/icons-svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { CatalogEntryStatus, ProviderStatusDto } from "../../lib/types";
  import {
    ACCENT_PRESETS,
    applyAccent,
    applyDeckTheme,
    applyMatchSystem,
    DECK_THEMES,
    loadAccent,
    loadCustomAccent,
    loadMatchSystem,
    loadTheme,
    MATCH_SYSTEM_STORAGE_KEY,
    syncSystemThemeListener,
    type AccentPreset,
    type DeckTheme,
  } from "../../lib/theme";
  import {
    loadSettingsSection,
    persistSettingsSection,
    type SettingsSectionId,
  } from "../../lib/navigation.svelte";
  import { setReducedMotion, setUiDensity, setUiScale, UI_SCALES, uiPrefs } from "../../lib/ui-prefs.svelte";
  import { ipc } from "../../lib/ipc";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  const VOICE_CAPTURE_KEY = "pytxo-desktop-voice-capture-v1";
  const VOICE_CONSENT_KEY = "pytxo-desktop-voice-cloud-consent-v1";
  const DEFAULT_PROFILE_KEY = "pytxo-default-permission-profile-v1";
  const OPEN_BEHAVIOR_KEY = "pytxo-workspace-open-behavior-v1";

  const SECTIONS: {
    id: SettingsSectionId;
    label: string;
    icon: typeof IconSettings;
  }[] = [
    { id: "general", label: "General", icon: IconSettings },
    { id: "appearance", label: "Appearance", icon: IconPalette },
    { id: "keyboard", label: "Keyboard", icon: IconKeyboard },
    { id: "providers", label: "Providers", icon: IconKey },
    { id: "workspaces", label: "Workspaces", icon: IconFolder },
    { id: "agents", label: "Agents & permissions", icon: IconShieldLock },
    { id: "voice", label: "Voice", icon: IconMicrophone },
    { id: "privacy", label: "Privacy", icon: IconBell },
    { id: "account", label: "Account & billing", icon: IconUserCircle },
  ];

  const PROFILES = [
    { id: "deep_space", label: "DeepSpace", hint: "Air-gapped cwd reads" },
    { id: "orbit", label: "Orbit", hint: "Default; approve-to-flush" },
    { id: "galaxy", label: "Galaxy", hint: "Host tools + HITL" },
    { id: "supernova", label: "Supernova", hint: "Full host privileges" },
  ] as const;

  let {
    initialSection = null,
    backend,
    activeDomain = null,
    tier,
    signedIn,
    cliMissing,
    subscriptionPortalUrl,
    voiceModelPath = null,
    voiceInstalling = false,
    onReplayOnboarding,
    onAuthChange = () => {},
    onInstallVoiceModel,
    onEditWorkspace,
    workspacesCatalog = undefined,
    agentsCatalog = undefined,
  }: {
    initialSection?: SettingsSectionId | null;
    backend: DesktopBackend;
    activeDomain?: CatalogEntryStatus | null;
    tier: string;
    signedIn: boolean;
    cliMissing: boolean;
    subscriptionPortalUrl: string | null;
    voiceModelPath?: string | null;
    voiceInstalling?: boolean;
    onReplayOnboarding: () => void;
    onAuthChange?: () => void;
    onInstallVoiceModel: () => void | Promise<void>;
    onEditWorkspace: (domainId: string) => void;
    /**
     * Workspaces and agent CLIs used to be separate destinations that also
     * appeared here. They now live only here, rendered inline so configuration
     * never sends the operator somewhere else to finish the same job.
     */
    workspacesCatalog?: Snippet;
    agentsCatalog?: Snippet;
  } = $props();

  let search = $state("");
  let section = $state<SettingsSectionId>(loadSettingsSection());
  let theme = $state<DeckTheme>(loadTheme());
  let accent = $state<AccentPreset>(loadAccent());
  let customAccent = $state(loadCustomAccent());
  let matchSystem = $state(loadMatchSystem());
  let voiceCapture = $state<"both" | "hold" | "click">("both");
  let voiceConsentNoted = $state(false);
  let showCloudConsent = $state(false);
  let defaultProfile = $state<string>("orbit");
  let openBehavior = $state<"last" | "picker">("last");
  let updateChecking = $state(false);
  let updateInstalling = $state(false);
  let pendingUpdate = $state<Update | null>(null);
  let updateMessage = $state("");
  let closeToTray = $state(true);
  let providers = $state<ProviderStatusDto[]>([]);
  let providersLoading = $state(false);
  let providersError = $state("");
  let showAllProviders = $state(false);
  let copiedProviderEnv = $state("");

  const filteredSections = $derived(
    SECTIONS.filter((s) => {
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      return s.label.toLowerCase().includes(q) || s.id.includes(q);
    }),
  );
  const configuredProviderCount = $derived(providers.filter((provider) => provider.key_configured).length);
  const visibleProviders = $derived.by(() => {
    const featured = ["deepseek", "openrouter", "openai", "anthropic"];
    const visible = showAllProviders
      ? providers
      : providers.filter((provider) => featured.includes(provider.id) || provider.key_configured);
    return [...visible].sort((a, b) => {
      const aPriority = featured.indexOf(a.id);
      const bPriority = featured.indexOf(b.id);
      if (aPriority >= 0 || bPriority >= 0) {
        return (aPriority < 0 ? 99 : aPriority) - (bPriority < 0 ? 99 : bPriority);
      }
      return a.name.localeCompare(b.name);
    });
  });

  $effect(() => {
    if (initialSection) {
      section = initialSection;
      persistSettingsSection(initialSection);
    }
  });

  onMount(() => {
    if (typeof localStorage === "undefined") return;
    const capture = localStorage.getItem(VOICE_CAPTURE_KEY);
    if (capture === "both" || capture === "hold" || capture === "click") voiceCapture = capture;
    voiceConsentNoted = localStorage.getItem(VOICE_CONSENT_KEY) === "true";
    const profile = localStorage.getItem(DEFAULT_PROFILE_KEY);
    if (profile && PROFILES.some((p) => p.id === profile)) defaultProfile = profile;
    const behavior = localStorage.getItem(OPEN_BEHAVIOR_KEY);
    if (behavior === "last" || behavior === "picker") openBehavior = behavior;
    void ipc.getCloseToTray().then((v) => {
      closeToTray = v;
    });
    void refreshProviders();
  });

  async function refreshProviders() {
    providersLoading = true;
    providersError = "";
    try {
      providers = await backend.listProviders();
    } catch (e) {
      providersError = e instanceof Error ? e.message : String(e);
      providers = [];
    } finally {
      providersLoading = false;
    }
  }

  function providerDetail(id: string): string {
    if (id === "deepseek") return "Metered DeepSeek API access. This is an API key, not a consumer login.";
    if (id === "openrouter") return "One metered key for multiple OpenAI-compatible models.";
    if (id === "openai") return "OpenAI API billing. ChatGPT subscription access stays with Codex.";
    if (id === "anthropic") return "Anthropic API billing. Claude subscription access stays with Claude Code.";
    return "Direct provider API access for explicitly selected runs.";
  }

  async function copyProviderVariable(variable: string) {
    if (!variable || typeof navigator === "undefined" || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(variable);
      copiedProviderEnv = variable;
      window.setTimeout(() => {
        if (copiedProviderEnv === variable) copiedProviderEnv = "";
      }, 1800);
    } catch {
      copiedProviderEnv = "";
    }
  }

  async function setCloseToTray(enabled: boolean) {
    closeToTray = enabled;
    try {
      await ipc.setCloseToTray(enabled);
    } catch {
      /* ignore when not in Tauri */
    }
  }

  function selectSection(id: SettingsSectionId) {
    section = id;
    persistSettingsSection(id);
  }

  function setTheme(next: DeckTheme) {
    matchSystem = false;
    syncSystemThemeListener(false);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(MATCH_SYSTEM_STORAGE_KEY, "false");
    }
    theme = next;
    applyDeckTheme(next);
  }

  function setAccent(next: AccentPreset) {
    accent = next;
    applyAccent(next, next === "custom" ? customAccent : undefined);
  }

  function setCustomAccent(hex: string) {
    customAccent = hex;
    accent = "custom";
    applyAccent("custom", hex);
  }

  function setMatchSystem(enabled: boolean) {
    matchSystem = enabled;
    if (enabled) {
      theme = applyMatchSystem(true);
      syncSystemThemeListener(true);
    } else {
      syncSystemThemeListener(false);
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(MATCH_SYSTEM_STORAGE_KEY, "false");
      }
      applyDeckTheme(theme);
    }
  }

  function setVoiceCapture(value: "both" | "hold" | "click") {
    voiceCapture = value;
    if (typeof localStorage !== "undefined") localStorage.setItem(VOICE_CAPTURE_KEY, value);
  }

  function setDefaultProfile(value: string) {
    defaultProfile = value;
    if (typeof localStorage !== "undefined") localStorage.setItem(DEFAULT_PROFILE_KEY, value);
  }

  function setOpenBehavior(value: "last" | "picker") {
    openBehavior = value;
    if (typeof localStorage !== "undefined") localStorage.setItem(OPEN_BEHAVIOR_KEY, value);
  }

  function consentCloudFallback() {
    voiceConsentNoted = true;
    if (typeof localStorage !== "undefined") localStorage.setItem(VOICE_CONSENT_KEY, "true");
    showCloudConsent = false;
  }

  async function signOut() {
    await ipc.authClearSession();
    onAuthChange();
  }

  function isBenignUpdaterError(message: string): boolean {
    const lower = message.toLowerCase();
    return (
      lower.includes("release json") ||
      lower.includes("could not fetch") ||
      lower.includes("network") ||
      lower.includes("404") ||
      lower.includes("not found") ||
      lower.includes("disabled")
    );
  }

  async function checkForUpdates() {
    updateChecking = true;
    updateMessage = "";
    pendingUpdate = null;
    try {
      const found = await check();
      if (found) {
        pendingUpdate = found;
        updateMessage = `Update available: v${found.version}`;
      } else {
        updateMessage = "You're on the latest version.";
      }
    } catch (e) {
      const msg = String(e);
      updateMessage = isBenignUpdaterError(msg)
        ? "No update channel available yet."
        : msg;
    } finally {
      updateChecking = false;
    }
  }

  async function installPendingUpdate() {
    if (!pendingUpdate) return;
    updateInstalling = true;
    try {
      await pendingUpdate.downloadAndInstall();
      await relaunch();
    } catch (e) {
      updateMessage = String(e);
      updateInstalling = false;
    }
  }
</script>

<section class="settings-screen">
  <div class="settings-rail">
    <label class="search">
      <IconSearch size={14} />
      <input bind:value={search} placeholder="Search settings" aria-label="Search settings" />
    </label>
    <nav aria-label="Settings sections">
      {#each filteredSections as item (item.id)}
        <button class:active={section === item.id} onclick={() => selectSection(item.id)}>
          <item.icon size={16} stroke={1.7} />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>
  </div>

  <div class="settings-main">
    <header class="screen-heading compact">
      <div>
        <!-- The page title is the destination, so a nav label always equals the
             title it leads to. The section is a subtitle inside it. -->
        <h1>Setup</h1>
        <h2 class="section-title">{SECTIONS.find((s) => s.id === section)?.label ?? "General"}</h2>
        <p>
          {#if section === "appearance"}Void, Light, scale, and density.
          {:else if section === "keyboard"}Chords for Work, Approvals, and the command palette.
          {:else if section === "providers"}Direct API access for selected runs. Agent subscription sessions live under Agents.
          {:else if section === "workspaces"}Trusted folders, defaults, and the active workspace.
          {:else if section === "agents"}Installed agent CLIs and the permission ladder default.
          {:else if section === "voice"}Local Whisper capture and optional cloud consent.
          {:else if section === "privacy"}What Pytxo stores and sanitizes locally.
          {:else if section === "account"}Account, tier, and onboarding.
          {:else}Startup density and motion preferences.{/if}
        </p>
      </div>
    </header>

    {#if section === "general"}
      <article class="settings-group">
        <h2>Startup</h2>
        <div class="setting-row">
          <div><strong>Density</strong><small>Compact fits more per screen; comfortable adds breathing room.</small></div>
          <div class="segmented">
            <button class:active={uiPrefs.density === "compact"} onclick={() => setUiDensity("compact")}>Compact</button>
            <button class:active={uiPrefs.density === "comfortable"} onclick={() => setUiDensity("comfortable")}>Comfortable</button>
          </div>
        </div>
        <div class="setting-row">
          <div><strong>Reduced motion</strong><small>Force off transitions and animations, regardless of OS setting.</small></div>
          <button class="toggle" class:active={uiPrefs.reducedMotion} aria-pressed={uiPrefs.reducedMotion} aria-label="Reduced motion" onclick={() => setReducedMotion(!uiPrefs.reducedMotion)}><i></i></button>
        </div>
        <div class="setting-row">
          <div><strong>Close to tray</strong><small>Keep Pytxo running when you close the window. Quit from the tray icon to exit.</small></div>
          <button
            class="toggle"
            class:active={closeToTray}
            aria-pressed={closeToTray}
            aria-label="Close to tray"
            onclick={() => void setCloseToTray(!closeToTray)}
          ><i></i></button>
        </div>
      </article>
    {:else if section === "appearance"}
      <article class="settings-group">
        <h2>Theme</h2>
        <div class="setting-row">
          <div><strong>Match system</strong><small>Follow OS light/dark. Uses Void and Light only.</small></div>
          <button
            class="toggle"
            class:active={matchSystem}
            aria-pressed={matchSystem}
            aria-label="Match system theme"
            onclick={() => setMatchSystem(!matchSystem)}
          ><i></i></button>
        </div>
        <div class="setting-row stack">
          <div><strong>Skin</strong><small>Void is canonical. Light is the other skin.</small></div>
          <div class="theme-previews" role="group" aria-label="Theme skin">
            {#each DECK_THEMES as opt (opt.id)}
              <button
                type="button"
                class="theme-preview"
                class:active={theme === opt.id}
                data-skin={opt.id}
                disabled={matchSystem && opt.id !== "void" && opt.id !== "light"}
                onclick={() => setTheme(opt.id)}
                aria-pressed={theme === opt.id}
                title={opt.hint}
              >
                <span class="theme-preview-swatch" aria-hidden="true"></span>
                <span class="theme-preview-label">{opt.label}</span>
              </button>
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div>
            <strong>Accent</strong>
            <small>
              Applies to focus rings and selection only. State colour is fixed: green means verified,
              red means refuted, violet means a decision is waiting.
            </small>
          </div>
          <div class="accent-swatches" role="group" aria-label="Accent">
            {#each ACCENT_PRESETS as opt (opt.id)}
              <button
                type="button"
                class="swatch"
                class:active={accent === opt.id}
                data-accent={opt.id}
                onclick={() => setAccent(opt.id)}
                title={opt.label}
                aria-label={opt.label}
                aria-pressed={accent === opt.id}
              ></button>
            {/each}
            <label
              class="swatch custom-swatch"
              class:active={accent === "custom"}
              title="Custom"
              aria-label="Custom accent color"
            >
              <input
                type="color"
                value={customAccent}
                oninput={(e) => setCustomAccent((e.currentTarget as HTMLInputElement).value)}
              />
            </label>
          </div>
        </div>
      </article>
      <article class="settings-group">
        <h2>Display</h2>
        <div class="setting-row">
          <div><strong>Scale</strong><small>Zooms the whole webview via Tauri, not just CSS.</small></div>
          <div class="segmented">
            {#each UI_SCALES as opt (opt.value)}
              <button class:active={uiPrefs.scale === opt.value} onclick={() => setUiScale(opt.value)}>{opt.label}</button>
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div><strong>Density</strong><small>Compact fits more per screen; comfortable adds breathing room.</small></div>
          <div class="segmented">
            <button class:active={uiPrefs.density === "compact"} onclick={() => setUiDensity("compact")}>Compact</button>
            <button class:active={uiPrefs.density === "comfortable"} onclick={() => setUiDensity("comfortable")}>Comfortable</button>
          </div>
        </div>
      </article>
    {:else if section === "keyboard"}
      <article class="settings-group">
        <h2>Shortcuts</h2>
        <div class="setting-row"><div><strong>Command palette</strong><small>Search destinations and New mission.</small></div><kbd>Ctrl/⌘ K</kbd></div>
        <div class="setting-row"><div><strong>Focus Operations</strong><small>Jump to today and the first running mission.</small></div><kbd>Ctrl/⌘ Shift O</kbd></div>
        <div class="setting-row"><div><strong>Approvals</strong><small>J and K move. Modifier Enter approves. Modifier Backspace denies.</small></div><kbd>J K</kbd></div>
        <div class="setting-row"><div><strong>Stop run</strong><small>From a focused running row. Confirmation is required.</small></div><kbd>Ctrl/⌘ Shift ⌫</kbd></div>
      </article>
    {:else if section === "providers"}
      <article class="settings-group">
        <h2>API providers</h2>
        <p class="providers-lead">
          API billing is separate from agent subscriptions. ChatGPT connects through Codex and
          Claude subscriptions connect through Claude Code in Agents. For direct API calls,
          set one provider variable in your OS environment and restart Desktop.
        </p>
        <div class="setting-row">
          <div><strong>{configuredProviderCount} configured</strong><small>Status is boolean only. Key values never enter Desktop.</small></div>
          <div class="provider-toolbar">
            <button class="quiet" onclick={() => (showAllProviders = !showAllProviders)}>{showAllProviders ? "Featured only" : `Show all ${providers.length}`}</button>
            <button class="quiet" onclick={() => void refreshProviders()} disabled={providersLoading}>
              {providersLoading ? "Refreshing…" : "Refresh"}
            </button>
          </div>
        </div>
        {#if providersError}
          <p class="providers-error">{providersError}</p>
        {/if}
        <div class="provider-list">
          {#each visibleProviders as p (p.id)}
            <div class="provider-row">
              <div>
                <strong>{p.name}</strong>
                <small>{providerDetail(p.id)}</small>
                <button class="provider-env mono" disabled={!p.api_key_env} onclick={() => void copyProviderVariable(p.api_key_env)}>
                  {p.api_key_env || "No key required"}{#if !p.builtin} · custom{/if}
                  {#if copiedProviderEnv === p.api_key_env && p.api_key_env}<span>Copied</span>{/if}
                </button>
              </div>
              <span class="provider-badge" class:ok={p.key_configured} class:missing={!p.key_configured}>
                {p.key_configured ? "Configured" : p.api_key_env ? "Not configured" : "Local"}
              </span>
            </div>
          {:else}
            <p class="providers-empty">{providersLoading ? "Loading…" : "No providers returned."}</p>
          {/each}
        </div>
        <p class="providers-docs">
          Docs: <span class="mono">pytxo.com/docs/reference/providers-byok</span>
          · CLI: <span class="mono">pytxo providers</span>
          · Custom endpoints: <span class="mono">~/.pytxo/providers.json</span>
        </p>
      </article>
    {:else if section === "workspaces"}
      {#if workspacesCatalog}<div class="embedded-catalog">{@render workspacesCatalog()}</div>{/if}
      <article class="settings-group">
        <h2>Defaults</h2>
        <div class="setting-row">
          <div><strong>On launch</strong><small>Restore the last active workspace or prompt to pick one.</small></div>
          <div class="segmented">
            <button class:active={openBehavior === "last"} onclick={() => setOpenBehavior("last")}>Last used</button>
            <button class:active={openBehavior === "picker"} onclick={() => setOpenBehavior("picker")}>Catalog</button>
          </div>
        </div>
      </article>
      <article class="settings-group">
        <h2>Active workspace</h2>
        {#if activeDomain}
          <div class="setting-row">
            <div>
              <strong>{activeDomain.repo_root.split(/[\\/]/).pop()}</strong>
              <small class="mono">{activeDomain.repo_root}</small>
            </div>
            <button class="quiet" onclick={() => onEditWorkspace(activeDomain.domain_id)}>Edit folders & permissions</button>
          </div>
        {:else}
          <div class="setting-row">
            <div><strong>No active workspace</strong><small>Add a folder in the catalog above to start a run.</small></div>
          </div>
        {/if}
      </article>
    {:else if section === "agents"}
      {#if agentsCatalog}<div class="embedded-catalog">{@render agentsCatalog()}</div>{/if}
      <article class="settings-group">
        <h2>Permission profile</h2>
        <div class="setting-row stack">
          <div><strong>Default for newly trusted folders</strong><small>Applied when you trust a workspace. Galaxy queues high-risk actions for HITL approval.</small></div>
          <div class="profile-grid">
            {#each PROFILES as p (p.id)}
              <button class:active={defaultProfile === p.id} onclick={() => setDefaultProfile(p.id)}>
                <strong>{p.label}</strong>
                <small>{p.hint}</small>
              </button>
            {/each}
          </div>
        </div>
      </article>
    {:else if section === "voice"}
      <article class="settings-group">
        <h2>Local transcription</h2>
        <div class="setting-row">
          <div><strong>Whisper model</strong><small>{voiceModelPath ?? "base.en · 148 MB · not installed"}</small></div>
          <button class="quiet" disabled={voiceInstalling} onclick={onInstallVoiceModel}>{voiceInstalling ? "Installing…" : "Install base.en"}</button>
        </div>
        <div class="setting-row">
          <div><strong>Capture behavior</strong><small>Click-to-record and press-and-hold; audio is never written to disk.</small></div>
          <select aria-label="Voice capture behavior" value={voiceCapture} onchange={(e) => setVoiceCapture(e.currentTarget.value as "both" | "hold" | "click")}>
            <option value="both">Both</option>
            <option value="hold">Press and hold</option>
            <option value="click">Click to record</option>
          </select>
        </div>
        <div class="setting-row">
          <div><strong>Cloud fallback</strong><small>{voiceConsentNoted ? "Consent noted for a future session (not used while local-only)." : "Requires explicit consent before any cloud transcription."}</small></div>
          <button class="quiet" onclick={() => (showCloudConsent = true)}>{voiceConsentNoted ? "Review again" : "Review consent"}</button>
        </div>
      </article>
    {:else if section === "privacy"}
      <article class="settings-group">
        <h2>Sovereign Shield</h2>
        <div class="setting-row"><div><strong>Raw audio retention</strong><small>Bounded memory only; cleared after completion, failure, or cancellation.</small></div><strong>Never stored</strong></div>
        <div class="setting-row"><div><strong>Transcript sanitization</strong><small>Sovereign Shield runs before cloud planning.</small></div><strong>Enabled</strong></div>
      </article>
    {:else if section === "account"}
      <article class="settings-group">
        <h2>Account</h2>
        <div class="setting-row"><div><strong>Status</strong><small>{signedIn ? "Signed in to your Pytxo account." : "Not signed in. Core local runs work without an account."}</small></div><strong>{signedIn ? "Signed in" : "Not signed in"}</strong></div>
        <div class="setting-row">
          <div><strong>Pytxo account connection</strong><small>{signedIn ? "Clear the existing local session stored in the OS keyring." : "Paused while the browser return moves to one-time authorization codes. Agent accounts are available in Agents."}</small></div>
          {#if signedIn}
            <button class="quiet" onclick={() => void signOut()}>Sign out</button>
          {:else}
            <strong>Local Core</strong>
          {/if}
        </div>
        <div class="setting-row"><div><strong>Tier</strong><small>Controls concurrent agent limits and cloud features.</small></div><strong class="mono">{tier}</strong></div>
        {#if signedIn && subscriptionPortalUrl}
          <div class="setting-row"><div><strong>Billing portal</strong><small>Manage plan, payment method, and invoices.</small></div><a class="quiet" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">Open portal</a></div>
        {/if}
        {#if cliMissing}
          <div class="setting-row"><div><strong>Pytxo CLI</strong><small>Not detected on PATH. Terminal parity and MCP tools need it.</small></div><strong>Missing</strong></div>
        {/if}
        <div class="setting-row">
          <div>
            <strong>Updates</strong>
            <small>{updateMessage || "Check GitHub releases for a newer Desktop build."}</small>
          </div>
          <div class="row-actions">
            <button class="quiet" disabled={updateChecking || updateInstalling} onclick={() => void checkForUpdates()}>
              {updateChecking ? "Checking…" : "Check for updates"}
            </button>
            {#if pendingUpdate}
              <button class="quiet" disabled={updateInstalling} onclick={() => void installPendingUpdate()}>
                {updateInstalling ? "Installing…" : "Restart to update"}
              </button>
            {/if}
          </div>
        </div>
        <div class="setting-row"><div><strong>Onboarding</strong><small>Replay the welcome flow, including CLI, account, and workspace checks.</small></div><button class="quiet" onclick={onReplayOnboarding}>Run onboarding again</button></div>
        <div class="setting-row"><div><strong>Community</strong><small>Ask questions, report issues, and follow releases with other Pytxo users.</small></div><a class="quiet" href="https://discord.gg/AUFRPFjSYv" target="_blank" rel="noopener noreferrer">Open Discord</a></div>
      </article>
    {/if}
  </div>

  {#if showCloudConsent}
    <div class="consent-sheet" role="dialog" aria-label="Cloud transcription consent">
      <p class="eyebrow">Per-session consent</p>
      <h2>Cloud transcription fallback</h2>
      <dl>
        <div><dt>Provider</dt><dd>Managed OpenAI transcription</dd></div>
        <div><dt>Audio destination</dt><dd>Provider processing endpoint</dd></div>
        <div><dt>Retention</dt><dd>Zero data retention where supported</dd></div>
      </dl>
      <p>Raw audio must be sent to the named provider for transcription. The returned transcript is sanitized before any later cloud planning. Local Whisper remains the default path.</p>
      <div>
        <button class="deny" onclick={() => (showCloudConsent = false)}>Cancel</button>
        <button class="primary" onclick={consentCloudFallback}>Consent for next session</button>
      </div>
    </div>
  {/if}
</section>

<style>
  .settings-screen {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    min-height: 100%;
    position: relative;
  }
  .settings-rail {
    border-right: 1px solid var(--pytxo-line, #1e2026);
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: color-mix(in oklab, var(--pytxo-graphite-1, #0d0f13) 80%, transparent);
  }
  /* Embedded catalogs are the section's real content, so they sit flush with the
     section body rather than reading as a nested screen. */
  .embedded-catalog {
    display: contents;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 7px;
    padding: 0 10px;
    background: #0a0b0e;
    color: #6b7280;
  }
  .search input {
    flex: 1;
    border: 0;
    background: transparent;
    color: #d5d9df;
    font-size: 12px;
    height: 32px;
    outline: none;
  }
  .settings-rail nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .settings-rail nav button {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 0;
    background: transparent;
    color: #8b929c;
    font-size: 12.5px;
    padding: 8px 10px;
    border-radius: 7px;
    cursor: pointer;
    text-align: left;
  }
  .settings-rail nav button:hover {
    background: #12141a;
    color: #d7dbe1;
  }
  .settings-rail nav button.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 14%, #12141a);
    color: #f1f3f5;
  }
  .settings-main {
    padding: 22px 28px 40px;
    max-width: 760px;
  }
  .screen-heading.compact {
    margin-bottom: 18px;
  }
  .screen-heading.compact h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }
  .screen-heading.compact .section-title {
    margin: 4px 0 0;
    color: var(--pytxo-text-strong, #f4f5f7);
    font-size: 14px;
    font-weight: 600;
  }
  .screen-heading.compact p {
    margin: 4px 0 0;
    color: #7a828e;
    font-size: 13px;
  }
  .settings-group {
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 10px;
    background: color-mix(in oklab, var(--pytxo-graphite-1, #0d0f13) 70%, transparent);
    margin-bottom: 14px;
    overflow: hidden;
  }
  .settings-group h2 {
    margin: 0;
    padding: 12px 16px 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #79828f;
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 16px;
    border-top: 1px solid color-mix(in oklab, var(--pytxo-line, #1e2026) 80%, transparent);
  }
  .setting-row.stack {
    flex-direction: column;
    align-items: stretch;
  }
  .setting-row strong {
    display: block;
    font-size: 13px;
    font-weight: 550;
    color: #e8eaed;
  }
  .setting-row small {
    display: block;
    margin-top: 3px;
    color: #79828f;
    font-size: 11.5px;
    line-height: 1.4;
  }
  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: flex-end;
  }
  .segmented {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border-radius: 7px;
    background: #0a0b0e;
    border: 1px solid var(--pytxo-line, #1e2026);
  }
  .segmented button {
    border: 0;
    background: transparent;
    color: #8b929c;
    font-size: 11.5px;
    padding: 6px 10px;
    border-radius: 5px;
    cursor: pointer;
  }
  .segmented button.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 18%, #161920);
    color: #f3f5f7;
  }
  .accent-swatches {
    display: flex;
    gap: 8px;
  }
  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
  }
  .swatch[data-accent="teal"] {
    background: var(--brand-teal);
  }
  .swatch.active {
    box-shadow: 0 0 0 2px color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 50%, transparent);
    border-color: #0a0b0e;
  }
  .custom-swatch {
    position: relative;
    overflow: hidden;
    background: conic-gradient(from 90deg, #f43f5e, #fbbf24, #22c55e, #22d3ee, #a78bfa, #f43f5e);
  }
  .custom-swatch input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
    width: 100%;
    height: 100%;
    border: 0;
    padding: 0;
  }
  .theme-previews {
    display: grid;
    grid-template-columns: repeat(2, minmax(72px, 1fr));
    gap: 8px;
    width: 100%;
    max-width: 420px;
  }
  .theme-preview {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: stretch;
    padding: 8px;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 8px;
    background: var(--pytxo-surface-panel, #0a0b0e);
    cursor: pointer;
    color: var(--pytxo-text-muted, #8b929d);
  }
  .theme-preview:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .theme-preview.active {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 45%, transparent);
    color: var(--pytxo-text-strong, #e8eaed);
  }
  .theme-preview-swatch {
    display: block;
    height: 28px;
    border-radius: 5px;
    border: 1px solid color-mix(in oklab, var(--pytxo-line) 80%, transparent);
  }
  .theme-preview[data-skin="void"] .theme-preview-swatch {
    background: linear-gradient(135deg, #020205, #0d0f13 55%, #2dd4bf);
  }
  .theme-preview[data-skin="light"] .theme-preview-swatch {
    background: linear-gradient(135deg, #f4f6f8, #ffffff 55%, #0d9488);
  }
  .theme-preview-label {
    font-size: 11px;
    text-align: left;
  }
  .setting-row.stack {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
    padding-block: 14px;
  }
  .setting-row.stack > div:first-child {
    width: 100%;
  }
  .providers-lead,
  .providers-docs,
  .providers-empty,
  .providers-error {
    margin: 0 0 12px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--pytxo-text-muted, #8b929d);
  }
  .providers-error {
    color: #d98994;
  }
  .provider-toolbar {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .provider-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  .provider-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 8px;
    background: var(--pytxo-surface-panel, #0a0b0e);
  }
  .provider-row > div {
    min-width: 0;
  }
  .provider-row strong {
    display: block;
    font-size: 12.5px;
  }
  .provider-row small {
    display: block;
    margin-top: 3px;
    color: var(--pytxo-text-muted, #8b929d);
    font-size: 11px;
  }
  .provider-env {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    margin-top: 7px;
    padding: 0;
    border: 0;
    background: transparent;
    color: #7e8792;
    font-size: 11px;
    cursor: pointer;
  }
  .provider-env:hover:not(:disabled) {
    color: #a4eee0;
  }
  .provider-env:focus-visible {
    outline: 2px solid var(--pytxo-accent, var(--pytxo-teal));
    outline-offset: 2px;
  }
  .provider-env:disabled {
    cursor: default;
  }
  .provider-env span {
    color: var(--pytxo-success, #52ba9a);
    font-family: "IBM Plex Sans", sans-serif;
  }
  .provider-badge {
    flex-shrink: 0;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid transparent;
  }
  .provider-badge.ok {
    color: var(--pytxo-success, #52ba9a);
    background: color-mix(in oklab, var(--pytxo-success, #52ba9a) 14%, transparent);
  }
  .provider-badge.missing {
    color: var(--pytxo-text-soft, #707783);
    background: color-mix(in oklab, var(--pytxo-text-soft, #707783) 12%, transparent);
  }
  .profile-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .profile-grid button {
    text-align: left;
    border: 1px solid var(--pytxo-line, #1e2026);
    background: #0a0b0e;
    border-radius: 8px;
    padding: 10px 12px;
    cursor: pointer;
    color: #c5cad1;
  }
  .profile-grid button.active {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 45%, transparent);
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 10%, #0a0b0e);
  }
  .profile-grid strong {
    font-size: 12.5px;
  }
  .profile-grid small {
    margin-top: 4px;
    color: #79828f;
    font-size: 11px;
  }
  .toggle {
    width: 36px;
    height: 20px;
    border-radius: 999px;
    border: 0;
    background: #2a2e36;
    position: relative;
    cursor: pointer;
    padding: 0;
  }
  .toggle i {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #c5cad1;
    transition: transform 120ms ease;
  }
  .toggle.active {
    background: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 70%, #1a3a34);
  }
  .toggle.active i {
    transform: translateX(16px);
    background: #fff;
  }
  .quiet,
  a.quiet {
    border: 1px solid var(--pytxo-line, #1e2026);
    background: #12141a;
    color: #c5cad1;
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    text-decoration: none;
    cursor: pointer;
  }
  .mono {
    font-family: "IBM Plex Mono", ui-monospace, monospace;
  }
  select {
    border: 1px solid var(--pytxo-line, #1e2026);
    background: #0a0b0e;
    color: #d5d9df;
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 12px;
  }
  .consent-sheet {
    position: absolute;
    inset: 0;
    z-index: 20;
    background: color-mix(in oklab, #07080b 88%, transparent);
    backdrop-filter: blur(8px);
    display: grid;
    place-content: center;
    padding: 24px;
  }
  .consent-sheet > :global(*) {
    max-width: 420px;
  }
  @media (max-width: 860px) {
    .settings-screen {
      grid-template-columns: 1fr;
    }
    .settings-rail {
      border-right: 0;
      border-bottom: 1px solid var(--pytxo-line, #1e2026);
    }
  }
</style>
