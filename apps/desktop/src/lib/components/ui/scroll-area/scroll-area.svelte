<script lang="ts">
  import { ScrollArea as ScrollAreaPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    class: className,
    children,
    ...restProps
  }: ScrollAreaPrimitive.RootProps & { children?: Snippet } = $props();
</script>

<ScrollAreaPrimitive.Root class={cn("relative overflow-hidden deck-scroll", className)} {...restProps}>
  <ScrollAreaPrimitive.Viewport class="h-full w-full rounded-[inherit]">
    {@render children?.()}
  </ScrollAreaPrimitive.Viewport>
  <ScrollAreaPrimitive.Scrollbar
    orientation="vertical"
    class="flex touch-none select-none p-px transition-colors"
  >
    <ScrollAreaPrimitive.Thumb class="relative flex-1 rounded-full bg-border" />
  </ScrollAreaPrimitive.Scrollbar>
</ScrollAreaPrimitive.Root>
