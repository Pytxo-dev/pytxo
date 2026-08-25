import { cn } from "@/lib/utils";

type ChassisShellProps = {
  children: React.ReactNode;
  className?: string;
};

export function ChassisShell({ children, className }: ChassisShellProps) {
  return (
    <div
      className={cn("relative isolate min-h-0 flex-1 overflow-x-clip chassis-shell", className)}
    >
      {children}
    </div>
  );
}

/** @deprecated Use ChassisShell. Kept so leftover imports fail loudly if missed. */
export function NebulaShell(props: ChassisShellProps) {
  return <ChassisShell {...props} />;
}
