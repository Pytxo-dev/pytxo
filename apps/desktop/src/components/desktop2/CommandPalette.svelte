<script lang="ts">
  import { IconSearch } from "@tabler/icons-svelte";
  import type { AppRoute } from "../../lib/navigation.svelte";

  export type CommandItem = {
    id: string;
    label: string;
    icon: typeof IconSearch;
    aliases?: string[];
    route?: AppRoute;
    run?: () => void;
    group?: "Navigate" | "Actions";
    hint?: string;
    disabled?: boolean;
  };

  let {
    open,
    items,
    onNavigate,
    onClose,
  }: {
    open: boolean;
    items: CommandItem[];
    onNavigate: (route: AppRoute) => void;
    onClose: () => void;
  } = $props();

  let dialogEl: HTMLDialogElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();
  let query = $state("");
  let highlighted = $state(0);

  const filtered = $derived(
    query.trim()
      ? items.filter((item) => {
          const q = query.trim().toLowerCase();
          return (
            item.label.toLowerCase().includes(q) ||
            (item.route ?? "").toLowerCase().includes(q) ||
            (item.aliases ?? []).some((alias) => alias.toLowerCase().includes(q))
          );
        })
      : items,
  );

  $effect(() => {
    if (!dialogEl) return;
    if (open) {
      query = "";
      highlighted = 0;
      if (!dialogEl.open) dialogEl.showModal();
      queueMicrotask(() => inputEl?.focus());
    } else if (dialogEl.open) {
      dialogEl.close();
    }
  });

  function select(item: CommandItem) {
    if (item.disabled) return;
    if (item.run) item.run();
    else if (item.route) onNavigate(item.route);
    onClose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      highlighted = filtered.length ? (highlighted + 1) % filtered.length : 0;
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      highlighted = filtered.length ? (highlighted - 1 + filtered.length) % filtered.length : 0;
    } else if (event.key === "Enter") {
      event.preventDefault();
      const item = filtered[highlighted];
      if (item) select(item);
    }
  }

  const groups = $derived(
    [...new Set(filtered.map((item) => item.group ?? "Navigate"))] as Array<
      "Navigate" | "Actions"
    >,
  );
</script>

<dialog
  bind:this={dialogEl}
  class="command-menu"
  aria-label="Command menu"
  onclose={onClose}
  onclick={(event) => {
    if (event.target === dialogEl) onClose();
  }}
>
  {#if open}
    <section>
      <label>
        <IconSearch size={18} />
        <input
          bind:this={inputEl}
          bind:value={query}
          oninput={() => (highlighted = 0)}
          onkeydown={onKeydown}
          placeholder="Type a command or search…"
          aria-label="Command search"
        />
      </label>
      {#if filtered.length}
        {#each groups as group (group)}
          <p>{group}</p>
          {#each filtered as item, index (`${item.id}`)}
            {#if (item.group ?? "Navigate") === group}
              <button
                class:highlighted={index === highlighted}
                class:disabled={item.disabled}
                disabled={item.disabled}
                onmouseenter={() => (highlighted = index)}
                onclick={() => select(item)}
              >
                <item.icon size={16} />{item.label}<span>{item.hint ?? (item.run ? "Run" : "Go to")}</span>
              </button>
            {/if}
          {/each}
        {/each}
      {:else}
        <p class="no-results">No matches for "{query}"</p>
      {/if}
    </section>
  {/if}
</dialog>

<style>
  .command-menu {
    position: fixed;
    top: 14vh;
    margin: 0 auto;
    width: min(560px, calc(100vw - 40px));
    padding: 8px;
    border: 1px solid #3d4450;
    border-radius: 4px;
    background: #1e232b;
    color: inherit;
    box-shadow: 0 24px 80px #000;
  }
  .command-menu::backdrop {
    background: rgba(0, 0, 0, 0.55);
  }
  .command-menu label {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px;
    border-bottom: 1px solid #3d4450;
    color: #7d8490;
  }
  .command-menu input {
    flex: 1;
    border: 0;
    outline: 0;
    background: none;
    color: #eef1f4;
    font: 13px inherit;
  }
  .command-menu p {
    margin: 11px 9px 5px;
    color: #5e6571;
    font-size: 12px;
  }
  .command-menu .no-results {
    margin: 4px 9px 13px;
    color: #767d89;
    font-size: 11px;
    text-transform: none;
    letter-spacing: normal;
  }
  .command-menu button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 34px;
    border: 0;
    border-radius: 2px;
    background: none;
    color: #aeb4be;
    font: 13px inherit;
    padding: 0 9px;
    cursor: pointer;
    transition: background-color 120ms ease, color 120ms ease;
  }
  .command-menu button:hover,
  .command-menu button.highlighted {
    background: #252b34;
    color: #fff;
  }
  .command-menu button.disabled,
  .command-menu button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .command-menu button:focus-visible {
    outline: 2px solid var(--live);
    outline-offset: -2px;
  }
  .command-menu button span {
    margin-left: auto;
    color: #565c68;
    font-family: "IBM Plex Mono", ui-monospace, monospace;
    font-size: 11px;
  }

  :global(html[data-chroma-theme="light"]) .command-menu {
    background: #fff;
    border-color: var(--pytxo-line);
    color: #17202b;
  }
</style>
