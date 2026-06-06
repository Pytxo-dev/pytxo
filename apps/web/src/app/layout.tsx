import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

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
    default: "Pytxo — Agent hypervisor & telemetry plane",
    template: "%s · Pytxo",
  },
  description:
    "High-velocity, low-overhead coordination of headless PTY agents with structural telemetry — not a cloud-heavy virtual workspace.",
  openGraph: {
    title: "Pytxo",
    description:
      "Agent hypervisor and telemetry plane for Claude Code, Codex, Antigravity CLI, and more.",
    url: "https://pytxo.com",
    siteName: "Pytxo",
    images: [{ url: "/logo.png", width: 512, height: 512, alt: "Pytxo" }],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Pytxo",
    description: "Agent hypervisor & telemetry plane",
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
        <TooltipProvider>{children}</TooltipProvider>
      </body>
    </html>
  );
}
