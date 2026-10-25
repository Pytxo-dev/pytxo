import { Callout } from "fumadocs-ui/components/callout";
import defaultMdxComponents from "fumadocs-ui/mdx";
import type { ComponentProps } from "react";
import type { MDXComponents } from "mdx/types";

const DocsLink = defaultMdxComponents.a;

// Evidence records such as /evidence/*.json are static files, not routes: a
// client-side link would prefetch them as pages and log a 404 on every view.
function MdxLink(props: ComponentProps<"a">) {
  const href = props.href ?? "";
  if (href.startsWith("/") && /\.\w+$/.test(href.split(/[?#]/, 1)[0])) {
    return <a {...props} />;
  }
  return <DocsLink {...props} />;
}

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    a: MdxLink,
    Callout,
    ...components,
  };
}
