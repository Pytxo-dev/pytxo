<script lang="ts">
  import { IconFolderPlus, IconTrash, IconX } from "@tabler/icons-svelte";
  import { ipc } from "../../lib/ipc";
  import type { CatalogEntryStatus, ProjectRootDto } from "../../lib/types";

  const PROFILES = [
    { id: "deep_space", label: "DeepSpace", hint: "Air-gapped cwd reads only" },
    { id: "orbit", label: "Orbit", hint: "Default; approve-to-flush sandbox" },
    { id: "galaxy", label: "Galaxy", hint: "Host tools; high-risk actions need HITL" },
    { id: "supernova", label: "Supernova", hint: "Full host user privileges" },
  ] as const;

  let {
    domain,
    onClose,
    onForgotten,
    onChanged,
  }: {
    domain: CatalogEntryStatus;
    onClose: () => void;
    onForgotten: (domainId: string) => void;
    onChanged?: () => void | Promise<void>;
  } = $props();

  let profile = $state("orbit");
  let trusted = $state(false);
  let roots = $state<ProjectRootDto[]>([]);
  let projectId = $state<string | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let forgetting = $state(false);
  let addingRoot = $state(false);

  const title = $derived(domain.repo_root.split(/[\\/]/).pop() ?? domain.domain_id);

  async function refresh() {
    loading = true;
    error = null;
    try {
      trusted = await ipc.domainIsTrusted(domain.repo_root);
      const current = await ipc.getDomainPermission(domain.repo_root);
      profile = current ?? (typeof localStorage !== "undefined"
        ? localStorage.getItem("pytxo-default-permission-profile-v1") ?? "orbit"
        : "orbit");
      projectId = domain.project_id ?? null;
      if (projectId) {
        roots = await ipc.projectRoots(projectId);
      } else {
        roots = [];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function saveProfile(next: string) {
    saving = true;
    error = null;
    try {
      profile = await ipc.setDomainPermission(domain.repo_root, next);
      trusted = true;
      await onChanged?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function addRoot() {
    if (!projectId || addingRoot) return;
    addingRoot = true;
    error = null;
    try {
      const picked = await ipc.pickWorkspaceFolder();
      if (!picked) return;
      roots = await ipc.projectAddRoot(projectId, picked, false);
      await onChanged?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      addingRoot = false;
    }
  }

  async function removeRoot(label: string) {
    if (!projectId) return;
    error = null;
    try {
      roots = await ipc.projectRemoveRoot(projectId, label);
      await onChanged?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function forget() {
    if (forgetting) return;
    forgetting = true;
    error = null;
    try {
      await ipc.forgetDomain(domain.domain_id);
      onForgotten(domain.domain_id);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      forgetting = false;
    }
  }

  $effect(() => {
    void refresh();
  });
</script>

<div class="ws-settings" role="dialog" aria-label={`Workspace settings · ${title}`}>
  <div class="sheet">
    <header>
      <div>
        <p class="eyebrow">Workspace settings</p>
        <h2>{title}</h2>
        <p class="path mono">{domain.repo_root}</p>
      </div>
      <button class="icon" onclick={onClose} aria-label="Close"><IconX size={16} /></button>
    </header>

    {#if loading}
      <p class="muted">Loading trust and folders…</p>
    {:else}
      <section>
        <h3>Permission profile</h3>
        <p class="hint">{trusted ? "Trusted on this machine." : "Not trusted yet. Choosing a profile trusts this folder for dispatch."}</p>
        <div class="profile-grid">
          {#each PROFILES as p (p.id)}
            <button class:active={profile === p.id} disabled={saving} onclick={() => saveProfile(p.id)}>
              <strong>{p.label}</strong>
              <small>{p.hint}</small>
            </button>
          {/each}
        </div>
      </section>

      <section>
        <div class="row-head">
          <h3>Folders</h3>
          {#if projectId}
            <button class="quiet" disabled={addingRoot} onclick={addRoot}>
              <IconFolderPlus size={14} /> {addingRoot ? "Picking…" : "Add folder"}
            </button>
          {/if}
        </div>
        {#if !projectId}
          <p class="hint">Single-root workspace. Create a modular project (multi-root) via the CLI to attach additional folders.</p>
          <div class="root-row">
            <div>
              <strong>Primary</strong>
              <small class="mono">{domain.repo_root}</small>
            </div>
            <span class="badge">primary</span>
          </div>
        {:else if roots.length}
          {#each roots as root (root.label + root.path)}
            <div class="root-row">
              <div>
                <strong>{root.label}</strong>
                <small class="mono">{root.path}</small>
              </div>
              <div class="root-meta">
                {#if root.primary}<span class="badge">primary</span>{/if}
                {#if root.read_only}<span class="badge">read-only</span>{/if}
                {#if !root.primary}
                  <button class="icon danger" aria-label={`Remove ${root.label}`} onclick={() => removeRoot(root.label)}>
                    <IconTrash size={14} />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        {:else}
          <p class="hint">No project roots recorded yet.</p>
        {/if}
      </section>

      <section class="danger">
        <h3>Danger</h3>
        <div class="root-row">
          <div>
            <strong>Forget workspace</strong>
            <small>Removes this domain from the local catalog. Does not delete files on disk.</small>
          </div>
          <button
            class="deny"
            disabled={forgetting || domain.active_runs > 0 || domain.hitl_pending > 0}
            onclick={forget}
          >
            {forgetting ? "Removing…" : domain.active_runs > 0 ? "Has active runs" : domain.hitl_pending > 0 ? "Has approvals" : "Forget"}
          </button>
        </div>
      </section>
    {/if}

    {#if error}<p class="error">{error}</p>{/if}
  </div>
</div>

<style>
  .ws-settings {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: color-mix(in oklab, #050608 72%, transparent);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .sheet {
    width: min(560px, 100%);
    max-height: min(86vh, 720px);
    overflow: auto;
    border: 1px solid var(--pytxo-line, #1e2026);
    border-radius: 12px;
    background: #0c0e13;
    padding: 18px 18px 20px;
    box-shadow: 0 24px 60px -28px rgba(0, 0, 0, 0.65);
  }
  header {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 16px;
  }
  .eyebrow {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #7d8591;
  }
  h2 {
    margin: 4px 0 0;
    font-size: 18px;
    font-weight: 600;
  }
  .path {
    margin: 4px 0 0;
    color: #7d8591;
    font-size: 11px;
  }
  section {
    margin-top: 16px;
    padding-top: 14px;
    border-top: 1px solid var(--pytxo-line, #1e2026);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 12px;
    font-weight: 600;
    color: #c5cad1;
  }
  .hint,
  .muted {
    margin: 0 0 10px;
    color: #7d8591;
    font-size: 12px;
    line-height: 1.4;
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
    display: block;
    font-size: 12.5px;
  }
  .profile-grid small {
    display: block;
    margin-top: 4px;
    color: #7d8591;
    font-size: 11px;
  }
  .row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }
  .root-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 0;
    border-top: 1px solid color-mix(in oklab, var(--pytxo-line, #1e2026) 70%, transparent);
  }
  .root-row strong {
    display: block;
    font-size: 12.5px;
  }
  .root-row small {
    display: block;
    margin-top: 3px;
    color: #7d8591;
    font-size: 11px;
  }
  .root-meta {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .badge {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 999px;
    border: 1px solid var(--pytxo-line, #1e2026);
    color: #8b929c;
  }
  .quiet,
  .deny,
  .icon {
    border: 1px solid var(--pytxo-line, #1e2026);
    background: #12141a;
    color: #c5cad1;
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .icon {
    width: 30px;
    height: 30px;
    padding: 0;
    justify-content: center;
  }
  .icon.danger:hover {
    color: #df6576;
  }
  .deny {
    border-color: #452a30;
    background: #1a1114;
    color: #d98a96;
  }
  .deny:disabled,
  .quiet:disabled,
  .profile-grid button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    margin: 12px 0 0;
    color: #d98a96;
    font-size: 12px;
  }
  .mono {
    font-family: "IBM Plex Mono", ui-monospace, monospace;
  }
  .danger h3 {
    color: #d98994;
  }
</style>
