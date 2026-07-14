import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";
import { NebulaShell } from "@/components/site/nebula-shell";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <NebulaShell className="flex h-dvh flex-col overflow-hidden">
      <SiteHeader />
      <main className="marketing-scroll relative flex min-h-0 flex-1 flex-col overflow-y-auto">
        {children}
        <SiteFooter />
      </main>
    </NebulaShell>
  );
}
