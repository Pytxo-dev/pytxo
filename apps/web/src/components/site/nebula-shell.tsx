import { cn } from "@/lib/utils";

type NebulaShellProps = {
  children: React.ReactNode;
  className?: string;
  subtle?: boolean;
};

export function NebulaShell({ children, className, subtle = false }: NebulaShellProps) {
  return (
    <div className={cn("relative isolate min-h-0 flex-1 overflow-x-clip", className)}>
      <div className="pointer-events-none absolute inset-0 -z-10 overflow-hidden" aria-hidden>
        <div
          className={cn(
            "absolute inset-0 nebula-bg",
            subtle && "opacity-70",
          )}
        />
        <div className="absolute inset-0 nebula-noise opacity-30 mix-blend-overlay" />
        <div
          className="absolute -top-24 left-1/4 size-72 rounded-full blur-3xl"
          style={{
            background:
              "radial-gradient(circle, color-mix(in oklab, var(--brand-violet) 20%, transparent) 0%, transparent 70%)",
          }}
        />
        <div
          className="absolute top-1/3 right-0 size-64 rounded-full blur-3xl"
          style={{
            background:
              "radial-gradient(circle, color-mix(in oklab, var(--brand-magenta) 16%, transparent) 0%, transparent 70%)",
          }}
        />
        <div
          className="absolute bottom-0 left-1/2 size-80 -translate-x-1/2 rounded-full blur-3xl"
          style={{
            background:
              "radial-gradient(circle, color-mix(in oklab, var(--brand-cyan) 12%, transparent) 0%, transparent 70%)",
          }}
        />
      </div>
      {children}
    </div>
  );
}
