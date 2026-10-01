import type { Metadata } from "next";
import { RootProvider } from "fumadocs-ui/provider/next";

import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://pytxo.com"),
  title: {
    default: "Pytxo: Local commit layer",
    template: "%s · Pytxo",
  },
  description:
    "A local commit layer around the coding-agent CLIs you already use. Isolate overlapping work, review the exact package, then Apply.",
  openGraph: {
    title: "Pytxo",
    description:
      "Local commit layer around the coding-agent CLIs you already use. Parallel runs, one reviewed Apply.",
    url: "https://pytxo.com",
    siteName: "Pytxo",
    images: [
      {
        url: "/product/work-1600x1000.png",
        width: 1600,
        height: 1000,
        alt: "Pytxo Desktop Work: the run ledger and the commit boundary",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: "Local commit layer around the coding-agent CLIs you already use",
    images: ["/product/work-1600x1000.png"],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased" suppressHydrationWarning>
      <head>
        <link rel="preconnect" href="https://api.fontshare.com" />
        <link
          rel="stylesheet"
          href="https://api.fontshare.com/v2/css?f[]=satoshi@400,500,600,700&display=swap"
        />
      </head>
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
