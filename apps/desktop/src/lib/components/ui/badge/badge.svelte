<script lang="ts" module>
  import { tv, type VariantProps } from "tailwind-variants";

  export const badgeVariants = tv({
    base: "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium transition-colors",
    variants: {
      variant: {
        default: "border-transparent bg-primary text-primary-foreground",
        secondary: "border-transparent bg-secondary text-secondary-foreground",
        outline: "text-foreground",
        muted: "border-transparent bg-muted text-muted-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  });

  export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  let {
    class: className,
    variant = "default",
    children,
    ...restProps
  }: HTMLAttributes<HTMLSpanElement> & {
    variant?: BadgeVariant;
    children?: Snippet;
  } = $props();
</script>

<span class={cn(badgeVariants({ variant }), className)} {...restProps}>
  {@render children?.()}
</span>
