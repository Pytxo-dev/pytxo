import type { NextConfig } from "next";
import { createMDX } from "fumadocs-mdx/next";

const withMDX = createMDX();

const nextConfig: NextConfig = {
  images: {
    unoptimized: true,
  },
  trailingSlash: false,
  reactStrictMode: true,
  allowedDevOrigins: ["127.0.0.1", "localhost"],
  // Pro checkout opens only with live Dodo keys: a test-mode checkout would
  // refuse real cards and grant Pro to anyone using a test card.
  env: {
    PYTXO_PRO_CHECKOUT_OPEN: process.env.DODO_ENVIRONMENT === "live" ? "1" : "",
  },
};

export default withMDX(nextConfig);
