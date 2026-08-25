import type { Metadata } from "next";
import { RootProvider } from "fumadocs-ui/provider/next";

import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://pytxo.com"),
  title: {
    default: "Pytxo: Run coding agents in parallel",
    template: "%s · Pytxo",
  },
  description:
    "Pytxo is a local agent hypervisor for Claude Code, Codex, and other coding agents. Parallel runs, collision-safe scheduling, approval gates. Not a cloud IDE.",
  openGraph: {
    title: "Pytxo",
    description:
      "A local agent hypervisor for the coding agents you already use, running in the background on your machine.",
    url: "https://pytxo.com",
    siteName: "Pytxo",
    images: [
      {
        url: "/product/operations-1600x1000.png",
        width: 1600,
        height: 1000,
        alt: "Pytxo Desktop Operations",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: "Local agent hypervisor for the coding agents you already use",
    images: ["/product/operations-1600x1000.png"],
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
              defaultTheme: "light",
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
