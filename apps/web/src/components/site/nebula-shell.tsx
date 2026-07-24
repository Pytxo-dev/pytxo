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
          className={cn("absolute inset-0 nebula-bg", subtle ? "opacity-40" : "opacity-70")}
        />
        <div className="absolute inset-0 nebula-noise opacity-10 mix-blend-overlay" />
      </div>
      {children}
    </div>
  );
}
