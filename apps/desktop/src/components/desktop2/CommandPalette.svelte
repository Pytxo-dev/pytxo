<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import IconSearch from "@tabler/icons-svelte/icons/search";
  import IconX from "@tabler/icons-svelte/icons/x";
  import type { AppRoute } from "../../lib/navigation.svelte";
  export type CommandItem = {
    id: string; label: string; icon: typeof IconSearch; aliases?: string[];
    route?: AppRoute; run?: () => void; group?: "Navigate" | "Actions" | "Settings";
    hint?: string; disabled?: boolean;
  };
  let { open, items, onNavigate, onClose }: {
    open: boolean; items: CommandItem[];
    onNavigate: (route: AppRoute) => void; onClose: () => void;
  } = $props();
  const menuId = $props.id();
  let dialogEl: HTMLDialogElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();
  let options: HTMLButtonElement[] = $state([]);
  let query = $state("");
  let highlighted = $state(0);
  const groupOrder = ["Actions", "Navigate", "Settings"] as const;
  const filtered = $derived.by(() => {
    const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return items.filter(item => {
      const text = [item.label, item.route ?? "", ...(item.aliases ?? [])].join(" ").toLowerCase();
      return words.every(word => text.includes(word));
    }).toSorted((a, b) => groupOrder.indexOf(a.group ?? "Navigate") - groupOrder.indexOf(b.group ?? "Navigate"));
  });
  const groups = $derived(groupOrder.filter(group => filtered.some(item => (item.group ?? "Navigate") === group)));
  const activeIndex = $derived(filtered[highlighted] && !filtered[highlighted].disabled
    ? highlighted : filtered.findIndex(item => !item.disabled));
  $effect(() => {
    if (!dialogEl) return;
    if (open) {
      query = ""; highlighted = 0;
      void withPreviewsHidden(() => { if (open && dialogEl?.isConnected) { if (!dialogEl.open) dialogEl.showModal(); inputEl?.focus(); } });
    } else if (dialogEl.open) dialogEl.close();
  });
  $effect(() => {
    const index = activeIndex;
    if (open && index >= 0) options[index]?.scrollIntoView({ block: "nearest" });
  });
  function select(item: CommandItem) {
    if (item.disabled) return;
    onClose();
    if (item.run) item.run(); else if (item.route) onNavigate(item.route);
  }
  function onKeydown(event: KeyboardEvent) {
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const enabled = filtered.map((item, index) => item.disabled ? -1 : index).filter(index => index >= 0);
      if (!enabled.length) return;
      const position = enabled.indexOf(activeIndex);
      highlighted = event.key === "Home" ? enabled[0] : event.key === "End" ? enabled[enabled.length - 1]
        : enabled[(position + (event.key === "ArrowDown" ? 1 : -1) + enabled.length) % enabled.length];
    } else if (event.key === "Enter") {
      event.preventDefault();
      const item = filtered[activeIndex];
      if (item) select(item);
    }
  }
</script>

<dialog bind:this={dialogEl} class="command-menu" aria-label="Command menu" onclose={onClose}
  onclick={(event) => { if (event.target === dialogEl) onClose(); }}>
  {#if open}
    <div class="command-search">
      <IconSearch size={18} />
      <input bind:this={inputEl} bind:value={query} oninput={() => (highlighted = 0)} onkeydown={onKeydown}
        placeholder="Search commands and settings…" aria-label="Command search" role="combobox"
        aria-expanded="true" aria-autocomplete="list" aria-controls={`${menuId}-results`}
        aria-activedescendant={activeIndex >= 0 ? `${menuId}-option-${filtered[activeIndex].id}` : undefined} />
      <button class="close-menu" aria-label="Close command menu" onclick={onClose}><IconX size={16} /></button>
    </div>
    <div class="command-results" id={`${menuId}-results`} role="listbox" aria-label="Commands">
      {#each groups as group (group)}
        <div role="group" aria-label={group}>
          <p class="group-label" aria-hidden="true">{group}</p>
          {#each filtered as item, index (item.id)}
            {#if (item.group ?? "Navigate") === group}
              <button bind:this={options[index]} id={`${menuId}-option-${item.id}`} role="option"
                aria-selected={index === activeIndex} aria-disabled={!!item.disabled}
                class:highlighted={index === activeIndex} disabled={item.disabled} tabindex="-1"
                onmouseenter={() => { if (!item.disabled) highlighted = index; }} onclick={() => select(item)}>
                <item.icon size={16} /><span class="command-label">{item.label}</span>
                {#if item.hint}<small title={item.hint}>{item.hint}</small>{/if}
              </button>
            {/if}
          {/each}
        </div>
      {/each}
    </div>
    {#if !filtered.length}
      <div class="no-results" role="status"><strong>No commands found</strong><p>Try a setting such as theme, an agent, or an action.</p><button onclick={() => { query = ""; inputEl?.focus(); }}>Clear search</button></div>
    {/if}
    <footer><span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span><span><kbd>Enter</kbd> Open</span><span><kbd>Esc</kbd> Close</span></footer>
  {/if}
</dialog>

<style>
  .command-menu {
    position: fixed; top: min(14vh, 100px); margin: 0 auto; padding: 0;
    width: min(600px, calc(100vw - 32px)); max-height: calc(100dvh - 120px);
    border: 1px solid var(--pytxo-line); border-radius: 12px;
    background: var(--pytxo-surface-panel); color: var(--pytxo-text-strong);
    box-shadow: 0 20px 80px -20px #0009, 0 4px 16px #0003;
  }
  .command-menu[open] { display: flex; flex-direction: column; overflow: hidden; }
  .command-menu::backdrop { background: #0008; }
  .command-search { flex: none; display: flex; align-items: center; gap: 12px; padding: 12px 14px; border-bottom: 1px solid var(--pytxo-line); color: var(--pytxo-text-muted); }
  input { min-width: 0; flex: 1; border: 0; outline: 0; background: transparent; color: var(--pytxo-text-strong); font: inherit; font-size: 14px; }
  input::placeholder { color: var(--pytxo-text-muted); }
  button { font: inherit; cursor: pointer; }
  .close-menu { display: grid; place-items: center; flex: none; width: 40px; height: 40px; border: 0; border-radius: 6px; color: var(--pytxo-text-soft); background: transparent; }
  .close-menu:hover { background: var(--pytxo-surface-hover); }
  .command-results { min-height: 0; max-height: min(420px, calc(100dvh - 270px)); overflow-y: auto; overscroll-behavior: contain; padding: 6px; scroll-padding-block: 6px; }
  .group-label { margin: 12px 10px 5px; color: var(--pytxo-text-muted); font-size: 11px; font-weight: 600; }
  .command-results button { display: flex; align-items: center; gap: 10px; width: 100%; min-height: 42px; padding: 8px 10px; border: 0; border-radius: 6px; background: transparent; color: var(--pytxo-text-body); text-align: left; font-size: 13px; transition: background-color 120ms ease, color 120ms ease; }
  .command-results button :global(svg) { flex: none; color: var(--pytxo-text-soft); }
  .command-label { min-width: 0; flex: 1; }
  .command-results button.highlighted { background: var(--pytxo-surface-active); color: var(--pytxo-text-strong); }
  .command-results button:disabled { color: var(--pytxo-text-muted); cursor: not-allowed; }
  .command-results small { max-width: 45%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; color: var(--pytxo-text-muted); }
  button:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -2px; }
  .no-results { padding: 24px; text-align: center; font-size: 13px; }
  .no-results p { color: var(--pytxo-text-muted); line-height: 1.6; }
  .no-results button { min-height: 40px; padding: 0 12px; border: 1px solid var(--pytxo-line); border-radius: 6px; color: var(--pytxo-text-strong); background: var(--pytxo-surface-raised); }
  footer { flex: none; display: flex; flex-wrap: wrap; gap: 16px; padding: 10px 16px; border-top: 1px solid var(--pytxo-line); color: var(--pytxo-text-muted); font-size: 11px; }
  footer span { display: inline-flex; align-items: center; gap: 5px; }
  kbd { padding: 1px 4px; border: 1px solid var(--pytxo-line); border-radius: 3px; font: inherit; }
  @media(max-height:600px){.command-menu{top:24px;max-height:calc(100dvh - 48px)}.command-results{max-height:none}}
</style>
