import type { Metadata } from "next";
import { RootProvider } from "fumadocs-ui/provider/next";

import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import { PRODUCT_SUMMARY, SHARE_IMAGE } from "@/lib/site";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://pytxo.com"),
  title: {
    default: "Pytxo: do your agents' branches pass together?",
    template: "%s · Pytxo",
  },
  description: PRODUCT_SUMMARY,
  openGraph: {
    title: "Pytxo",
    description: PRODUCT_SUMMARY,
    url: "https://pytxo.com",
    siteName: "Pytxo",
    images: [SHARE_IMAGE],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: PRODUCT_SUMMARY,
    images: [SHARE_IMAGE.url],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased" suppressHydrationWarning>
      <body className="flex min-h-full flex-col">
        <Providers>
          <RootProvider
            theme={{
              defaultTheme: "dark",
              enableSystem: false,
            }}
          >
            <TooltipProvider>{children}</TooltipProvider>
          </RootProvider>
        </Providers>
      </body>
    </html>
  );
}
