<script lang="ts">
  import type { Snippet } from "svelte";
  import { onDestroy } from "svelte";
  import { cn } from "$lib/utils";

  let {
    open = $bindable(false),
    align = "end" as "start" | "end",
    class: className,
    trigger,
    children,
  }: {
    open?: boolean;
    align?: "start" | "end";
    class?: string;
    trigger: Snippet<[{ toggle: () => void; open: boolean }]>;
    children: Snippet;
  } = $props();

  let rootEl: HTMLDivElement | undefined = $state();

  function toggle() {
    open = !open;
  }

  function onDocPointer(e: PointerEvent) {
    if (!open || !rootEl) return;
    if (!rootEl.contains(e.target as Node)) open = false;
  }

  function onDocKey(e: KeyboardEvent) {
    if (e.key === "Escape" && open) open = false;
  }

  $effect(() => {
    if (!open) return;
    document.addEventListener("pointerdown", onDocPointer);
    document.addEventListener("keydown", onDocKey);
    return () => {
      document.removeEventListener("pointerdown", onDocPointer);
      document.removeEventListener("keydown", onDocKey);
    };
  });

  onDestroy(() => {
    document.removeEventListener("pointerdown", onDocPointer);
    document.removeEventListener("keydown", onDocKey);
  });
</script>

<div class={cn("relative inline-flex", className)} bind:this={rootEl}>
  {@render trigger({ toggle, open })}
  {#if open}
    <div
      class="absolute z-50 mt-1 min-w-[8rem] overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
      class:right-0={align === "end"}
      class:left-0={align === "start"}
      role="menu"
    >
      {@render children()}
    </div>
  {/if}
</div>
