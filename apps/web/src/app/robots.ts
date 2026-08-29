import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
    },
    sitemap: "https://pytxo.com/sitemap.xml",
    host: "https://pytxo.com",
  };
}
