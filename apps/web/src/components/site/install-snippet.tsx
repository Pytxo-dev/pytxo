import { cn } from "@/lib/utils";

type InstallSnippetProps = {
  children: string;
  className?: string;
};

/** Shared mono install / command block — one visual language site-wide. */
export function InstallSnippet({ children, className }: InstallSnippetProps) {
  return (
    <pre
      className={cn(
        "overflow-x-auto rounded-[var(--radius-md)] border border-border bg-background/80 px-4 py-3 font-mono text-sm text-foreground/90",
        className,
      )}
    >
      {children}
    </pre>
  );
}
