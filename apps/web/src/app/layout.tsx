import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

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
    "Coordinate Claude Code, Codex, and other agents locally. Parallel runs, collision-safe scheduling, optional Desktop change map. Not a cloud IDE.",
  openGraph: {
    title: "Pytxo",
    description:
      "Run and coordinate the coding agents you already use, on your machine, in the background.",
    url: "https://pytxo.com",
    siteName: "Pytxo",
    images: [{ url: "/logo.png", width: 512, height: 512, alt: "Pytxo" }],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: "Local coordinator for coding agents you already use",
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
    >
      <body className="flex min-h-full flex-col">
        <Providers>
          <TooltipProvider>{children}</TooltipProvider>
        </Providers>
      </body>
    </html>
  );
}
