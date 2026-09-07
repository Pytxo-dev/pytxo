import type { Metadata } from "next";

/** Each public page keeps its canonical and share identity together. */
export function pageMetadata(path: string, title: string, description: string): Metadata {
  const image = {
    url: "/product/work-1600x1000.png",
    width: 1600,
    height: 1000,
    alt: "Pytxo Desktop Work: the run ledger and the commit boundary",
  };

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
