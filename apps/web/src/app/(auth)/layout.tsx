import type { ReactNode } from "react";
import Link from "next/link";

export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div
      className="chassis-shell dark flex min-h-dvh flex-col overflow-y-auto bg-background text-foreground"
      data-chroma-theme="void"
    >
      <header className="flex items-center justify-between px-4 py-4 sm:px-6">
        <Link href="/" className="text-sm font-semibold tracking-tight text-foreground">
          Pytxo
        </Link>
        <Link
          href="/download"
          className="text-sm text-muted-foreground transition-colors hover:text-foreground"
        >
          Download
        </Link>
      </header>
      <div className="flex flex-1 flex-col items-center justify-center px-4 py-12">{children}</div>
    </div>
  );
}
