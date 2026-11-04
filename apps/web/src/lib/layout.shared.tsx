import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import Image from "next/image";

import { DISCORD_URL, NAV_LINKS } from "@/lib/site";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <Image src="/logo-mark.png" alt="" width={22} height={22} className="size-[22px]" />
          Pytxo
        </>
      ),
      url: "/docs",
    },
    links: [...NAV_LINKS.filter((l) => l.href !== "/docs"), { href: DISCORD_URL, label: "Support" }].map((l) => ({
      text: l.label,
      url: l.href,
      active: "none" as const,
    })),
  };
}
