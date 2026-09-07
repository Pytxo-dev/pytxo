import type { MetadataRoute } from "next";

import { assertDocsSource } from "@/lib/docs";

const ORIGIN = "https://pytxo.com";

export default function sitemap(): MetadataRoute.Sitemap {
  const docs = assertDocsSource().map((page) => ({
    url: `${ORIGIN}${page.url}`,
    changeFrequency: "weekly" as const,
    priority: page.url === "/docs" ? 0.9 : 0.7,
  }));

  return [
    { url: ORIGIN, changeFrequency: "weekly", priority: 1 },
    { url: `${ORIGIN}/download`, changeFrequency: "weekly", priority: 0.8 },
    { url: `${ORIGIN}/evidence`, changeFrequency: "weekly", priority: 0.8 },
    { url: `${ORIGIN}/plans`, changeFrequency: "monthly", priority: 0.6 },
    ...docs,
  ];
}
