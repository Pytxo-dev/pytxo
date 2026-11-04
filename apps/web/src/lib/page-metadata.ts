import type { Metadata } from "next";
import { SHARE_IMAGE } from "@/lib/site";

/** Each public page keeps its canonical and share identity together. */
export function pageMetadata(path: string, title: string, description: string): Metadata {
  const image = SHARE_IMAGE;

  return {
    title: { absolute: title },
    description,
    alternates: { canonical: path },
    openGraph: {
      title,
      description,
      url: path,
      siteName: "Pytxo",
      images: [image],
      locale: "en_US",
      type: "website",
    },
    twitter: {
      card: "summary_large_image",
      title,
      description,
      images: [image],
    },
  };
}
