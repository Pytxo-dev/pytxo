<script lang="ts">
  import { onMount } from "svelte";
  import {
    IconArrowLeft,
    IconBell,
    IconFolder,
    IconMicrophone,
    IconPalette,
    IconSearch,
    IconSettings,
    IconShieldLock,
    IconUserCircle,
  } from "@tabler/icons-svelte";
  import type { CatalogEntryStatus } from "../../lib/types";
  import {
    ACCENT_PRESETS,
    applyAccent,
    applyDeckTheme,
    DECK_THEMES,
    loadAccent,
    loadTheme,
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
    activeDomain = null,
    tier,
    signedIn,
    cliMissing,
    subscriptionPortalUrl,
    voiceModelPath = null,
    voiceInstalling = false,
    onBack,
    onReplayOnboarding,
    onAuthChange = () => {},
    onInstallVoiceModel,
    onEditWorkspace,
    onOpenWorkspaces,
  }: {
    initialSection?: SettingsSectionId | null;
    activeDomain?: CatalogEntryStatus | null;
    tier: string;
    signedIn: boolean;
    cliMissing: boolean;
    subscriptionPortalUrl: string | null;
    voiceModelPath?: string | null;
    voiceInstalling?: boolean;
    onBack: () => void;
    onReplayOnboarding: () => void;
    onAuthChange?: () => void;
    onInstallVoiceModel: () => void | Promise<void>;
    onEditWorkspace: (domainId: string) => void;
    onOpenWorkspaces: () => void;
  } = $props();

  let search = $state("");
  let section = $state<SettingsSectionId>(initialSection ?? loadSettingsSection());
  let theme = $state<DeckTheme>(loadTheme());
  let accent = $state<AccentPreset>(loadAccent());
  let voiceCapture = $state<"both" | "hold" | "click">("both");
  let voiceConsentNoted = $state(false);
  let showCloudConsent = $state(false);
  let defaultProfile = $state<string>("orbit");
  let openBehavior = $state<"last" | "picker">("last");
  let updateChecking = $state(false);
  let updateInstalling = $state(false);
  let pendingUpdate = $state<Update | null>(null);
  let updateMessage = $state("");

  const filteredSections = $derived(
    SECTIONS.filter((s) => {
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      return s.label.toLowerCase().includes(q) || s.id.includes(q);
    }),
  );

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
  });

  function selectSection(id: SettingsSectionId) {
    section = id;
    persistSettingsSection(id);
  }

  function setTheme(next: DeckTheme) {
    theme = next;
    applyDeckTheme(next);
  }

  function setAccent(next: AccentPreset) {
    accent = next;
    applyAccent(next);
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

  async function signIn() {
    await ipc.authOpenSignIn();
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
    <button class="back" onclick={onBack}><IconArrowLeft size={15} /> Back</button>
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
        <h1>{SECTIONS.find((s) => s.id === section)?.label ?? "Settings"}</h1>
        <p>
          {#if section === "appearance"}Theme skins and chroma accents for the shell.
          {:else if section === "workspaces"}Default workspace behavior and the active domain.
          {:else if section === "agents"}Permission ladder defaults for new trusted folders.
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
      </article>
    {:else if section === "appearance"}
      <article class="settings-group">
        <h2>Theme</h2>
        <div class="setting-row">
          <div><strong>Skin</strong><small>Void is canonical. Terminal and Nebula are optional skins.</small></div>
          <div class="segmented wrap">
            {#each DECK_THEMES as opt (opt.id)}
              <button class:active={theme === opt.id} onclick={() => setTheme(opt.id)}>{opt.label}</button>
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div><strong>Accent</strong><small>Hairlines, focus rings, and primary edges follow this preset.</small></div>
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
      </article>
    {:else if section === "workspaces"}
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
            <div><strong>No active workspace</strong><small>Open a folder from the Workspaces catalog.</small></div>
            <button class="quiet" onclick={onOpenWorkspaces}>Open catalog</button>
          </div>
        {/if}
      </article>
    {:else if section === "agents"}
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
        <div class="setting-row"><div><strong>Status</strong><small>{signedIn ? "Signed in to your Pytxo account." : "Running in local mode; no account linked."}</small></div><strong>{signedIn ? "Signed in" : "Local mode"}</strong></div>
        <div class="setting-row">
          <div><strong>Sign in</strong><small>{signedIn ? "Clear the local session stored in the OS keyring." : "Opens pytxo.com in your browser, then returns via deep link."}</small></div>
          {#if signedIn}
            <button class="quiet" onclick={() => void signOut()}>Sign out</button>
          {:else}
            <button class="quiet" onclick={() => void signIn()}>Sign in</button>
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
  .back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    background: transparent;
    color: #8b929c;
    font-size: 12px;
    cursor: pointer;
    padding: 6px 8px;
    border-radius: 6px;
    width: fit-content;
  }
  .back:hover {
    background: #14161c;
    color: #e8eaed;
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
  .screen-heading.compact p {
    margin: 6px 0 0;
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
    color: #6f7784;
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
    color: #6f7784;
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
  .segmented.wrap {
    flex-wrap: wrap;
    max-width: 320px;
    justify-content: flex-end;
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
  .swatch[data-accent="spectrum"] {
    background: linear-gradient(135deg, var(--brand-magenta), var(--brand-violet), var(--brand-gold), var(--brand-teal));
  }
  .swatch[data-accent="teal"] {
    background: var(--brand-teal);
  }
  .swatch[data-accent="violet"] {
    background: var(--brand-violet);
  }
  .swatch[data-accent="gold"] {
    background: var(--brand-gold);
  }
  .swatch.active {
    box-shadow: 0 0 0 2px color-mix(in oklab, var(--pytxo-accent, var(--pytxo-teal)) 50%, transparent);
    border-color: #0a0b0e;
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
    color: #6f7784;
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
    font-family: "Geist Mono", ui-monospace, monospace;
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
