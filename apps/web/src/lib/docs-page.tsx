import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
} from "fumadocs-ui/layouts/docs/page";
import { createRelativeLink } from "fumadocs-ui/mdx";
import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { getMDXComponents } from "@/components/mdx";
import { source } from "@/lib/source";
import { pageMetadata } from "@/lib/page-metadata";

export async function renderDocsPage(slug: string[]) {
  const page = source.getPage(slug);
  if (!page) notFound();

  const MDX = page.data.body;

  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      <DocsBody>
        <MDX
          components={getMDXComponents({
            a: createRelativeLink(source, page),
          })}
        />
      </DocsBody>
    </DocsPage>
  );
}

export function docsMetadata(slug: string[]): Metadata {
  const page = source.getPage(slug);
  if (!page) notFound();
  return pageMetadata(
    page.url,
    `${page.data.title} · Pytxo`,
    page.data.description ?? `Pytxo documentation: ${page.data.title}.`,
  );
}
