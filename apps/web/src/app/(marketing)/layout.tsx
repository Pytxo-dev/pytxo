import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="chassis-shell dark min-h-dvh bg-background text-foreground" data-chroma-theme="void">
      <SiteHeader />
      <main className="marketing-scroll">
        {children}
        <SiteFooter />
      </main>
    </div>
  );
}
