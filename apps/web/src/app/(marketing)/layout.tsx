import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";
import { NebulaShell } from "@/components/site/nebula-shell";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <NebulaShell className="flex min-h-svh flex-col overflow-x-clip">
      <SiteHeader />
      <main className="relative flex flex-1 flex-col">{children}</main>
      <SiteFooter />
    </NebulaShell>
  );
}
