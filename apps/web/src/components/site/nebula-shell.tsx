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
          className={cn("absolute inset-0", subtle ? "opacity-35" : "opacity-55")}
          style={{
            background:
              "radial-gradient(ellipse 70% 45% at 88% -10%, color-mix(in oklab, var(--brand-teal) 8%, transparent), transparent 55%), radial-gradient(ellipse 55% 40% at 8% 0%, color-mix(in oklab, var(--brand-violet) 6%, transparent), transparent 52%), radial-gradient(ellipse 40% 30% at 70% 20%, color-mix(in oklab, var(--brand-gold) 4%, transparent), transparent 48%)",
          }}
        />
        <div className="absolute inset-0 nebula-noise opacity-8 mix-blend-overlay" />
      </div>
      {children}
    </div>
  );
}
