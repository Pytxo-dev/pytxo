<script lang="ts">
  import ProfileIcon from "./ProfileIcon.svelte";
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import { onMount, type Snippet } from "svelte";
  import IconBell from "@tabler/icons-svelte/icons/bell";
  import IconFolder from "@tabler/icons-svelte/icons/folder";
  import IconKey from "@tabler/icons-svelte/icons/key";
  import IconKeyboard from "@tabler/icons-svelte/icons/keyboard";
  import IconMicrophone from "@tabler/icons-svelte/icons/microphone";
  import IconPalette from "@tabler/icons-svelte/icons/palette";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import IconSettings from "@tabler/icons-svelte/icons/settings";
  import IconShieldLock from "@tabler/icons-svelte/icons/shield-lock";
  import IconUserCircle from "@tabler/icons-svelte/icons/user-circle";
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
  import { SETTINGS_GROUPS, SETTINGS_SECTIONS, settingsSectionMatches } from "../../lib/settings-catalog";
  import { setReducedMotion, setUiDensity, setUiScale, UI_SCALES, uiPrefs } from "../../lib/ui-prefs.svelte";
  import { ipc, onAuthChanged } from "../../lib/ipc";
  import { MOD_KEY } from "../../lib/platform";
  import UpdateControls from "../shell/UpdateControls.svelte";

  const VOICE_CAPTURE_KEY = "pytxo-desktop-voice-capture-v1";
  const VOICE_CONSENT_KEY = "pytxo-desktop-voice-cloud-consent-v1";
  const DEFAULT_PROFILE_KEY = "pytxo-default-permission-profile-v1";
  const OPEN_BEHAVIOR_KEY = "pytxo-workspace-open-behavior-v1";

  const SECTION_ICONS: Record<SettingsSectionId, typeof IconSettings> = {
    general: IconSettings,
    appearance: IconPalette,
    keyboard: IconKeyboard,
    workspaces: IconFolder,
    agents: IconShieldLock,
    providers: IconKey,
    voice: IconMicrophone,
    privacy: IconBell,
    account: IconUserCircle,
  };
  const SECTIONS = SETTINGS_SECTIONS.map((item) => ({ ...item, icon: SECTION_ICONS[item.id] }));

  const PROFILES = [
    { id: "deep_space", label: "DeepSpace", hint: "Air-gapped cwd reads" },
    { id: "orbit", label: "Orbit", hint: "Review before Apply" },
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
  let searchInput: HTMLInputElement | undefined = $state();
  let section = $state<SettingsSectionId>(loadSettingsSection());
  let settingsHeading: HTMLElement | undefined = $state();
  $effect(() => {
    if (section && settingsHeading) settingsHeading.closest<HTMLElement>(".settings-main")?.scrollTo({ top: 0, behavior: "instant" });
  });
  let theme = $state<DeckTheme>(loadTheme());
  let accent = $state<AccentPreset>(loadAccent());
  let customAccent = $state(loadCustomAccent());
  let matchSystem = $state(loadMatchSystem());
  let voiceCapture = $state<"both" | "hold" | "click">("both");
  let voiceConsentNoted = $state(false);
  let showCloudConsent = $state(false);
  let consentDialog: HTMLDialogElement | undefined = $state();
  let cancelConsent: HTMLButtonElement | undefined = $state();
  $effect(() => {
    if (showCloudConsent && consentDialog && !consentDialog.open) {
      void withPreviewsHidden(() => { if (showCloudConsent && consentDialog?.isConnected) { if (!consentDialog.open) consentDialog.showModal(); cancelConsent?.focus(); } });
    }
  });
  let defaultProfile = $state<string>("orbit");
  let openBehavior = $state<"last" | "picker">("last");
  let closeToTray = $state(true);
  let providers = $state<ProviderStatusDto[]>([]);
  let providersLoading = $state(false);
  let providersError = $state("");
  let showAllProviders = $state(false);
  let copiedProviderEnv = $state("");
  let routingCredentialPresent = $state(false);
  let routingBridgeAvailable = $state(false);
  let routingAccountId = $state<string | null>(null);
  let routingRemoteStatus = $state<"absent" | "verified" | "unverified" | "revoked">("absent");
  let routingRecoveryOnly = $state(false);
  let routingAccountMessage = $state("");

  const searchQuery = $derived(search.trim());
  const filteredSections = $derived(SECTIONS.filter((item) => settingsSectionMatches(item, search)));
  const sectionGroups = $derived(
    SETTINGS_GROUPS.map((label) => ({ label, items: filteredSections.filter((item) => item.group === label) }))
      .filter((group) => group.items.length > 0),
  );
  const currentSection = $derived(SECTIONS.find((item) => item.id === section) ?? SECTIONS[0]!);
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
    let active = true;
    let unlisten: (() => void) | null = null;
    const verifyTimer = setInterval(() => void refreshRoutingAccount(), 30_000);
    void refreshRoutingAccount();
    void onAuthChanged(() => void refreshRoutingAccount()).then((stop) => {
      if (active) unlisten = stop;
      else stop();
    });
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
    return () => { active = false; unlisten?.(); clearInterval(verifyTimer); };
  });

  async function refreshRoutingAccount() {
    const status = await ipc.routingAccountStatus().catch(() => null);
    routingCredentialPresent = status?.credential_present ?? false;
    routingBridgeAvailable = status?.bridge_available ?? false;
    routingAccountId = status?.account_id ?? null;
    routingRemoteStatus = status?.remote_status ?? "absent";
    routingRecoveryOnly = status?.recovery_only ?? false;
  }

  async function connectRoutingAccount() {
    routingAccountMessage = "";
    try {
      await ipc.routingAccountConnect();
    } catch {
      routingAccountMessage = "Could not open the account connection. Try again.";
    }
  }

  async function disconnectRoutingAccount() {
    if (!window.confirm("Revoke all Routing access for this account? This disconnects every Desktop and disables all hosted workspace grants and evaluation tokens.")) return;
    routingAccountMessage = "";
    try {
      const result = await ipc.routingAccountDisconnect();
      await refreshRoutingAccount();
      routingAccountMessage = result.remote_revoked
        ? "Routing access revoked for this account."
        : "Local credential cleared. Remote revocation could not be confirmed; use your web account to revoke all routing access.";
    } catch (cause) {
      routingAccountMessage = cause && typeof cause === "object" && "message" in cause
        ? String(cause.message)
        : "Could not revoke this account's Routing access.";
    }
  }

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

  function clearSearch() {
    search = "";
    searchInput?.focus();
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

</script>

<div class="settings-viewport">
<section class="settings-screen">
  <div class="settings-rail">
    <div class="search">
      <IconSearch size={14} aria-hidden="true" />
      <input bind:this={searchInput} bind:value={search} placeholder="Search settings" aria-label="Search settings" />
      {#if search}<button class="search-clear" aria-label="Clear settings search" onclick={clearSearch}>Clear</button>{/if}
    </div>
    {#if searchQuery}
      <p class="search-summary" role="status">
        {filteredSections.length ? `${filteredSections.length} ${filteredSections.length === 1 ? "section matches" : "sections match"}` : "No settings found"}
        <span>“{searchQuery}”</span>
      </p>
      {#if !filteredSections.length}<p class="search-help">Try a control name, such as scale, updates, or microphone.</p>{/if}
    {/if}
    <label class="compact-section">
      <span>Section</span>
      <select aria-label="Settings section" value={section} onchange={event => selectSection(event.currentTarget.value as SettingsSectionId)}>
        {#each sectionGroups as group (group.label)}
          <optgroup label={group.label}>
            {#each group.items as item (item.id)}<option value={item.id}>{item.label}</option>{/each}
          </optgroup>
        {/each}
      </select>
    </label>
    <nav aria-label="Settings sections">
      {#each sectionGroups as group (group.label)}
        <div class="nav-group" role="group" aria-label={group.label}>
          <p class="nav-group-label">{group.label}</p>
          {#each group.items as item (item.id)}
            <button class:active={section === item.id} aria-current={section === item.id ? "page" : undefined} aria-label={item.label} aria-describedby={searchQuery ? `settings-match-${item.id}` : undefined} onclick={() => selectSection(item.id)}>
              <item.icon size={16} stroke={1.7} aria-hidden="true" />
              <span class="nav-copy">
                <span>{item.label}</span>
                {#if searchQuery}<small id={`settings-match-${item.id}`}>{item.description}</small>{/if}
              </span>
            </button>
          {/each}
        </div>
      {/each}
    </nav>
  </div>

  <div class="settings-main">
    <div class="settings-content">
    <header class="settings-heading" bind:this={settingsHeading}>
      <div>
        <h1>Setup</h1>
        <h2 class="section-title">{currentSection.label}</h2>
        <p>{currentSection.description}</p>
      </div>
    </header>

    {#if section === "general"}
      <article class="settings-group">
        <h2>Window & motion</h2>
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
      <article class="settings-group">
        <h2>Desktop updates</h2>
        <div class="updater-settings"><UpdateControls /></div>
      </article>
    {:else if section === "appearance"}
      <article class="settings-group">
        <h2>Theme</h2>
        <div class="setting-row">
          <div><strong>Match system</strong><small>Follow your computer’s light or dark appearance.</small></div>
          <button
            class="toggle"
            class:active={matchSystem}
            aria-pressed={matchSystem}
            aria-label="Match system theme"
            onclick={() => setMatchSystem(!matchSystem)}
          ><i></i></button>
        </div>
        <div class="setting-row stack">
          <div><strong>Theme</strong><small>Choose a dark or light workspace.</small></div>
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
                aria-label="Custom accent color"
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
          <div><strong>Scale</strong><small>Adjust the size of text, icons, and controls together.</small></div>
          <div class="segmented">
            {#each UI_SCALES as opt (opt.value)}
              <button class:active={uiPrefs.scale === opt.value} aria-pressed={uiPrefs.scale === opt.value} onclick={() => setUiScale(opt.value)}>{opt.label}</button>
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div><strong>Density</strong><small>Compact fits more per screen; comfortable adds breathing room.</small></div>
          <div class="segmented">
            <button class:active={uiPrefs.density === "compact"} aria-pressed={uiPrefs.density === "compact"} onclick={() => setUiDensity("compact")}>Compact</button>
            <button class:active={uiPrefs.density === "comfortable"} aria-pressed={uiPrefs.density === "comfortable"} onclick={() => setUiDensity("comfortable")}>Comfortable</button>
          </div>
        </div>
      </article>
    {:else if section === "keyboard"}
      <article class="settings-group">
        <h2>Shortcuts</h2>
        <div class="setting-row"><div><strong>Command palette</strong><small>Find actions, destinations, and settings.</small></div><kbd>{MOD_KEY} K</kbd></div>
        <div class="setting-row"><div><strong>Focus Work</strong><small>Open Work and focus the active run.</small></div><kbd>{MOD_KEY} Shift O</kbd></div>
        <div class="setting-row"><div><strong>Approvals</strong><small>J and K move. Modifier Enter approves. Modifier Backspace denies.</small></div><kbd>J K</kbd></div>
        <div class="setting-row"><div><strong>Stop run</strong><small>From a focused running row. Confirmation is required.</small></div><kbd>{MOD_KEY} Shift ⌫</kbd></div>
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
            <button class:active={openBehavior === "last"} aria-pressed={openBehavior === "last"} onclick={() => setOpenBehavior("last")}>Last used</button>
            <button class:active={openBehavior === "picker"} aria-pressed={openBehavior === "picker"} onclick={() => setOpenBehavior("picker")}>Catalog</button>
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
        <h2>Default permissions</h2>
        <div class="setting-row stack">
          <div><strong>{PROFILES.find(p => p.id === defaultProfile)?.label ?? "Orbit"} for new folders</strong><small>Orbit is the beta starting point. Existing workspace permissions stay unchanged.</small></div>
          <details class="permission-options">
            <summary>Change default permissions</summary>
          <div class="profile-grid">
            {#each PROFILES as p (p.id)}
              <button class:active={defaultProfile === p.id} aria-pressed={defaultProfile === p.id} onclick={() => setDefaultProfile(p.id)}>
                <ProfileIcon profile={p.id} /><strong>{p.label}</strong>
                <small>{p.hint}</small>
              </button>
            {/each}
          </div>
          </details>
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
        <div class="setting-row">
          <div><strong>Experimental Routing account</strong><small>{routingRecoveryOnly ? "Recovery connection: inspect and revoke existing hosted grants in Work. New grants are unavailable through this connection." : !routingBridgeAvailable ? routingCredentialPresent ? "New account connections are off. Revoking here affects every Desktop and hosted workspace grant on this account." : "The account bridge is off in this build. Local Core remains available." : routingRemoteStatus === "verified" ? `Routing-only connection verified for ${routingAccountId ?? "this account"}. Hosted packets still require a separate workspace opt-in.` : routingRemoteStatus === "revoked" ? "This Routing credential was revoked. Revoke any saved hosted grants in Work before reconnecting." : routingCredentialPresent ? "A Routing credential is stored locally, but Link could not verify it. Hosted Routing is unavailable until verification succeeds." : "Connect your Pytxo account in the browser for a routing-only credential. Agent and Ultra billing remain separate."}</small></div>
          <button class="quiet" disabled={!routingBridgeAvailable && !routingCredentialPresent} onclick={() => void (routingRecoveryOnly && routingBridgeAvailable ? connectRoutingAccount() : routingCredentialPresent ? disconnectRoutingAccount() : connectRoutingAccount())}>{routingRecoveryOnly && routingBridgeAvailable ? "Reconnect full Routing access" : routingCredentialPresent ? "Revoke all Routing access" : "Connect"}</button>
        </div>
        {#if routingAccountMessage}<p role="status">{routingAccountMessage}</p>{/if}
        <div class="setting-row"><div><strong>Tier</strong><small>Controls concurrent agent limits and cloud features.</small></div><strong class="mono">{tier}</strong></div>
        {#if signedIn && subscriptionPortalUrl}
          <div class="setting-row"><div><strong>Billing portal</strong><small>Manage plan, payment method, and invoices.</small></div><a class="quiet" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">Open portal</a></div>
        {/if}
        {#if cliMissing}
          <div class="setting-row"><div><strong>Pytxo CLI</strong><small>Not detected on PATH. Terminal parity and MCP tools need it.</small></div><strong>Missing</strong></div>
        {/if}
        <div class="setting-row"><div><strong>Onboarding</strong><small>Replay the welcome flow, including CLI, account, and workspace checks.</small></div><button class="quiet" onclick={onReplayOnboarding}>Run onboarding again</button></div>
        <div class="setting-row"><div><strong>Community</strong><small>Ask questions, report issues, and follow releases with other Pytxo users.</small></div><a class="quiet" href="https://discord.gg/AUFRPFjSYv" target="_blank" rel="noopener noreferrer">Open Discord</a></div>
      </article>
    {/if}
    </div>
  </div>

  {#if showCloudConsent}
    <dialog bind:this={consentDialog} class="consent-sheet" aria-label="Cloud transcription consent"
      onclose={() => (showCloudConsent = false)} onkeydown={(event) => {
        if (event.key !== "Tab" || !consentDialog) return;
        const buttons = Array.from(consentDialog.querySelectorAll<HTMLButtonElement>("button"));
        event.preventDefault();
        buttons[(buttons.indexOf(document.activeElement as HTMLButtonElement) + 1) % buttons.length]?.focus();
      }}>
      <p class="eyebrow">Per-session consent</p>
      <h2>Cloud transcription fallback</h2>
      <dl>
        <div><dt>Provider</dt><dd>Managed OpenAI transcription</dd></div>
        <div><dt>Audio destination</dt><dd>Provider processing endpoint</dd></div>
        <div><dt>Retention</dt><dd>Zero data retention where supported</dd></div>
      </dl>
      <p>Raw audio must be sent to the named provider for transcription. The returned transcript is sanitized before any later cloud planning. Local Whisper remains the default path.</p>
      <div class="consent-actions">
        <button bind:this={cancelConsent} class="quiet" onclick={() => consentDialog?.close()}>Cancel</button>
        <button class="primary" onclick={consentCloudFallback}>Consent for next session</button>
      </div>
    </dialog>
  {/if}
</section>
</div>

<style>
  .settings-viewport { container-type: inline-size; min-width: 0; min-height: 0; flex: 1; overflow: hidden; }
  .compact-section { display: none; }
  .settings-screen {
    display: grid;
    /* Wide enough that "Agents & permissions" stays on one line. */
    grid-template-columns: 212px minmax(0, 1fr);
    height: 100%;
    min-height: 0;
    grid-template-rows: minmax(0, 1fr);
    position: relative;
  }
  .settings-rail {
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
    box-sizing: border-box;
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
  .embedded-catalog + .settings-group {
    margin-top: 14px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 7px;
    padding: 0 10px;
    background: var(--pytxo-surface-input);
    color: var(--pytxo-text-muted);
  }
  .search input {
    flex: 1;
    border: 0;
    background: transparent;
    color: var(--pytxo-text-strong);
    font-size: 12px;
    height: 40px;
    min-width: 0;
    outline: none;
  }
  .search-clear {
    flex: none;
    min-height: 40px;
    border: 0;
    padding: 0 2px;
    background: transparent;
    color: var(--pytxo-text-body);
    font-size: 11px;
    cursor: pointer;
  }
  .search-clear:hover { color: var(--pytxo-text-strong); }
  .search-summary,
  .search-help {
    margin: 0;
    color: var(--pytxo-text-muted);
    font-size: 11px;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }
  .search-summary span {
    display: block;
    color: var(--pytxo-text-body);
  }
  .settings-rail nav {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .nav-group-label {
    margin: 0 10px 4px;
    color: var(--pytxo-text-muted);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .settings-rail nav button {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 0;
    background: transparent;
    color: var(--pytxo-text-soft);
    font-size: 12.5px;
    padding: 8px 10px;
    border-radius: 7px;
    cursor: pointer;
    text-align: left;
    min-height: 40px;
    transition: background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), color var(--pytxo-motion-fast) var(--pytxo-motion-ease);
  }
  .settings-rail nav button :global(svg) { flex: none; }
  .nav-copy {
    display: grid;
    gap: 4px;
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .nav-copy small {
    color: var(--pytxo-text-muted);
    font-size: 11px;
    line-height: 1.5;
  }
  .settings-rail nav button:hover {
    background: var(--pytxo-surface-hover);
    color: var(--pytxo-text-strong);
  }
  .settings-rail nav button.active {
    background: var(--pytxo-surface-active);
    color: var(--pytxo-text-strong);
    box-shadow: inset 2px 0 var(--pytxo-text-strong);
    font-weight: 600;
  }
  .settings-main {
    container-type: inline-size;
    width: 100%;
    box-sizing: border-box;
    min-width: 0;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
    scroll-padding-block: 24px;
  }
  .settings-content {
    width: 100%;
    max-width: 960px;
    box-sizing: border-box;
    padding: 24px 12px 32px clamp(16px, 3cqi, 32px);
  }
  .settings-heading {
    margin-bottom: 18px;
  }
  .settings-heading h1 {
    margin: 0;
    color: var(--pytxo-text-muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .settings-heading .section-title {
    margin: 6px 0 0;
    color: var(--pytxo-text-strong, #f4f5f7);
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.025em;
    line-height: 1.25;
  }
  .settings-heading p {
    margin: 4px 0 0;
    color: var(--pytxo-text-muted);
    font-size: 13px;
    line-height: 1.5;
  }
  /* Each group is a terminal pane with its name cut into the top border. */
  .settings-group {
    position: relative;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 6px;
    background: transparent;
    margin: 22px 0 14px;
    padding-top: 6px;
  }
  .settings-group h2 {
    position: absolute;
    top: -9px;
    left: 12px;
    margin: 0;
    padding: 0 7px;
    background: var(--pytxo-surface-shell);
    font: 600 11px/18px var(--pytxo-font-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--pytxo-text-soft);
  }
  .settings-group > h2 + * { border-top: 0; }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 16px;
    border-top: 1px solid color-mix(in oklab, var(--pytxo-line, #1e2026) 80%, transparent);
  }
  .updater-settings { padding: 12px 16px; border-top: 1px solid var(--pytxo-line); }
  .setting-row.stack {
    flex-direction: column;
    align-items: stretch;
  }
  .permission-options { width: 100%; }
  .permission-options > summary { cursor: pointer; padding: 8px 0; font-size: 13px; color: var(--pytxo-text-soft); }
  .setting-row strong {
    display: block;
    font-size: 14px;
    font-weight: 550;
    color: var(--pytxo-text-strong);
  }
  .setting-row small {
    display: block;
    margin-top: 3px;
    color: var(--pytxo-text-muted);
    font-size: 13px;
    line-height: 1.55;
  }
  .settings-screen .segmented {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    border-radius: 9px;
    background: var(--pytxo-surface-input);
    flex: none;
    border: 1px solid var(--pytxo-line, #1e2026);
  }
  .settings-screen .segmented button {
    border: 0;
    background: transparent;
    color: var(--pytxo-text-muted);
    font-size: 12px;
    min-height: 40px;
    padding: 0 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), color var(--pytxo-motion-fast) var(--pytxo-motion-ease);
  }
  .settings-screen .segmented button.active {
    background: var(--pytxo-surface-active);
    color: var(--pytxo-text-strong);
  }
  .accent-swatches {
    display: flex;
    gap: 8px;
    flex: none;
  }
  .swatch {
    width: 40px;
    height: 40px;
    border-radius: 999px;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 7px;
    background-clip: content-box;
  }
  .swatch[data-accent="teal"] {
    background-color: var(--pytxo-text-strong);
  }
  .swatch.active {
    box-shadow: 0 0 0 2px color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 50%, transparent);
    border-color: var(--pytxo-text-muted);
  }
  .custom-swatch {
    position: relative;
    overflow: hidden;
    background: conic-gradient(from 90deg, #f43f5e, #fbbf24, #22c55e, #22d3ee, #a78bfa, #f43f5e);
    background-clip: content-box;
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
    position: relative;
    height: 64px;
    border-radius: 5px;
    border: 1px solid color-mix(in oklab, var(--pytxo-line) 80%, transparent);
  }
  .theme-preview[data-skin="void"] .theme-preview-swatch {
    background: linear-gradient(90deg, #17171b 0 22%, transparent 22%), linear-gradient(#17171b 0 10px, #08080a 10px);
    color: #595961;
  }
  .theme-preview[data-skin="light"] .theme-preview-swatch {
    background: linear-gradient(90deg, #e8ebef 0 22%, transparent 22%), linear-gradient(#e8ebef 0 10px, #fff 10px);
    color: #abb1bb;
  }
  .theme-preview-swatch::after{content:"";position:absolute;left:30%;top:23px;width:50%;height:3px;background:currentColor;border-radius:2px;box-shadow:0 9px 0 currentColor,0 18px 0 currentColor;opacity:.5}
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
    margin: 0 16px 12px;
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
    color: var(--pytxo-text-soft);
    font-size: 12px;
    cursor: pointer;
  }
  .provider-env:hover:not(:disabled) {
    color: var(--pytxo-text-strong);
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
    background: var(--pytxo-surface-input);
    border-radius: 8px;
    padding: 10px 12px;
    cursor: pointer;
    color: var(--pytxo-text-body);
  }
  .profile-grid button.active {
    border-color: color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 45%, transparent);
    background: var(--pytxo-surface-active);
  }
  .profile-grid strong {
    font-size: 12.5px;
  }
  .profile-grid small {
    margin-top: 4px;
    color: var(--pytxo-text-muted);
    font-size: 11px;
  }
  .settings-screen .toggle {
    width: 44px;
    height: 40px;
    flex: none;
    border-radius: 6px;
    border: 0;
    background: transparent;
    position: relative;
    cursor: pointer;
    padding: 0;
  }
  .toggle::before{content:"";position:absolute;inset:10px 4px;border-radius:10px;background:var(--pytxo-line);transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .settings-screen .toggle i {
    position: absolute;
    top: 12px;
    left: 6px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--pytxo-text-muted);
    transition: transform var(--pytxo-motion-fast) var(--pytxo-motion-ease), background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease);
  }
  .settings-screen .toggle.active {
    background: transparent;
  }
  .settings-screen .toggle:hover,
  .settings-screen .toggle.active:hover { background: transparent; }
  .settings-screen .toggle:hover::before { background: color-mix(in oklab, var(--pytxo-line) 75%, var(--pytxo-text-muted)); }
  .settings-screen .toggle.active:hover::before { background: var(--pytxo-text-body); }
  .settings-screen .toggle.active::before{background:var(--pytxo-text-strong)}
  .settings-screen .toggle.active i {
    transform: translateX(16px);
    background: var(--pytxo-surface-shell);
  }
  .search:focus-within,
  .custom-swatch:focus-within { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  button:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .segmented button:hover:not(.active) { background: var(--pytxo-surface-hover); color: var(--pytxo-text-strong); }
  .quiet,
  a.quiet {
    border: 1px solid var(--pytxo-line, #1e2026);
    background: var(--pytxo-surface-raised);
    color: var(--pytxo-text-strong);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    text-decoration: none;
    cursor: pointer;
    min-height: 40px;
  }
  .mono {
    font-family: "IBM Plex Mono", ui-monospace, monospace;
  }
  select {
    border: 1px solid var(--pytxo-line, #1e2026);
    background: var(--pytxo-surface-input);
    color: var(--pytxo-text-strong);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 12px;
  }
  .consent-sheet {
    width: min(480px, calc(100vw - 32px));
    box-sizing: border-box;
    max-height: calc(100dvh - 48px);
    overflow: auto;
    margin: auto;
    border: 1px solid var(--pytxo-line);
    border-radius: 12px;
    color: var(--pytxo-text-strong);
    background: var(--pytxo-surface-panel);
    padding: 24px;
    box-shadow: 0 24px 80px -24px #0009;
  }
  .consent-sheet::backdrop { background: #0008; }
  .consent-sheet h2 { margin: 0 0 20px; font-size: 20px; letter-spacing: -.025em; }
  .consent-sheet dl { margin: 0; }
  .consent-sheet dl > div { display: grid; grid-template-columns: 110px minmax(0, 1fr); gap: 12px; padding: 10px 0; border-bottom: 1px solid var(--pytxo-line-soft); font-size: 12px; line-height: 1.5; }
  .consent-sheet dt { color: var(--pytxo-text-muted); }
  .consent-sheet dd { margin: 0; }
  .consent-sheet p { color: var(--pytxo-text-muted); font-size: 13px; line-height: 1.6; }
  .consent-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; margin-top: 20px; }
  .consent-actions button { min-height: 40px; }
  .consent-sheet > :global(*) {
    max-width: 420px;
  }
  @container (max-width: 720px) {
    .settings-screen { grid-template-columns: minmax(0,1fr); grid-template-rows: auto minmax(0,1fr); }
    .settings-rail { position: static; height: auto; display: grid; grid-template-columns: minmax(0,1fr) minmax(160px,1fr); align-items: end; border-right: 0; border-bottom: 1px solid var(--pytxo-line); padding: 12px 16px; }
    .settings-rail nav { display: none; }
    .compact-section { display: flex; flex-direction: column; gap: 4px; color: var(--pytxo-text-muted); font-size: 11px; min-width: 0; }
    .compact-section select { min-width: 0; width: 100%; min-height: 40px; padding: 0 28px 0 10px; border: 1px solid var(--pytxo-line); border-radius: 6px; background-color: var(--pytxo-surface-input); color: var(--pytxo-text-strong); font: inherit; font-size: 13px; }
    .search-summary, .search-help { grid-column: 1/-1; }
    .settings-content { padding: 20px 16px; }
  }
  @container (max-width: 440px) {
    .setting-row { flex-wrap: wrap; gap: 10px; }
    .setting-row > div:first-child { flex: 1 1 180px; min-width: 0; }
    .settings-group .segmented { flex-wrap: wrap; max-width: 100%; }
    .accent-swatches { flex-wrap: wrap; }
  }
</style>
