import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { RootProvider } from "fumadocs-ui/provider/next";

import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

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
        url: "/logo.png",
        width: 2000,
        height: 2000,
        alt: "Pytxo",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: "Local agent hypervisor for the coding agents you already use",
    images: ["/logo.png"],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} dark h-full antialiased`}
      suppressHydrationWarning
    >
      <body className="flex min-h-full flex-col">
        <Providers>
          <RootProvider>
            <TooltipProvider>{children}</TooltipProvider>
          </RootProvider>
        </Providers>
      </body>
    </html>
  );
}
