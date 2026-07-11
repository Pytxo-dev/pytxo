<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  let {
    class: className,
    selected = false,
    children,
    ...restProps
  }: HTMLButtonAttributes & {
    selected?: boolean;
    children?: Snippet;
  } = $props();
</script>

<button
  type="button"
  role="menuitem"
  class={cn(
    "flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent hover:text-accent-foreground",
    selected && "bg-accent/60 text-primary",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</button>
