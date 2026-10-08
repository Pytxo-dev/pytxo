import type { Metadata } from "next";
import type { ReactNode } from "react";
import Image from "next/image";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Sign in",
  description: "Sign in to your Pytxo account.",
  robots: { index: false, follow: false },
};

export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div
      className="chassis-shell dark flex min-h-dvh flex-col overflow-y-auto bg-background text-foreground"
      data-chroma-theme="void"
    >
      <header className="flex items-center justify-between px-4 py-4 sm:px-6">
        <Link href="/" className="flex items-center gap-2.5 text-[15px] font-semibold tracking-tight text-foreground">
          <Image src="/logo-mark.png" alt="" width={22} height={22} priority />
          Pytxo
        </Link>
        <Link
          href="/download"
          className="text-sm text-muted-foreground transition-colors hover:text-foreground"
        >
          Download
        </Link>
      </header>
      <div className="flex flex-1 flex-col items-center justify-center gap-6 px-4 py-12">
        {children}
        <p className="max-w-sm text-center text-xs leading-relaxed text-muted-foreground">
          You don&apos;t need an account to use Pytxo on your machine. Your coding agents keep
          their own sign-in.
        </p>
      </div>
    </div>
  );
}
