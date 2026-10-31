import { SiteFooter } from "@/components/site/footer";
import { SiteHeader } from "@/components/site/header";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div
      className="chassis-shell dark flex min-h-dvh flex-col bg-background text-foreground"
      data-chroma-theme="void"
      data-site="v2"
    >
      <SiteHeader />
      <main id="main" className="w-full max-w-full flex-1 overflow-x-hidden">
        {children}
      </main>
      <SiteFooter />
    </div>
  );
}
