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
          className={cn("absolute inset-0", subtle ? "opacity-50" : "opacity-80")}
          style={{
            background:
              "radial-gradient(ellipse 80% 50% at 50% -20%, color-mix(in oklab, var(--brand-violet) 12%, transparent), transparent 55%), radial-gradient(ellipse 50% 40% at 90% 10%, color-mix(in oklab, var(--brand-teal) 8%, transparent), transparent 50%)",
          }}
        />
        <div className="absolute inset-0 nebula-noise opacity-10 mix-blend-overlay" />
      </div>
      {children}
    </div>
  );
}
