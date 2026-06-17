import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  images: {
    unoptimized: true,
  },
  trailingSlash: false,
  async rewrites() {
    return {
      beforeFiles: [
        {
          source: "/docs/:path((?!assets/)(?!img/).*)",
          destination: "/docs/:path.html",
        },
      ],
    };
  },
};

export default nextConfig;
