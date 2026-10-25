"use client";

import Image from "next/image";
import Link from "next/link";
import { MenuIcon } from "lucide-react";

import { HeaderAuth } from "@/components/header-auth";
import { Button } from "@/components/ui/button";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { NAV_LINKS } from "@/lib/site";
import { cn } from "@/lib/utils";

export function SiteHeader() {
  const desktopLinks = NAV_LINKS.filter((link) => link.href !== "/download");

  return (
    <header
      data-site-header
      className="sticky top-0 z-50 border-b border-white/[0.04] bg-[#050507]/90 backdrop-blur-xl"
    >
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-3 focus:z-10 focus:rounded-[4px] focus:bg-white focus:px-3 focus:py-2 focus:text-sm focus:font-medium focus:text-black"
      >
        Skip to content
      </a>
      <div className="mx-auto grid h-16 max-w-[92rem] grid-cols-[1fr_auto_1fr] items-center gap-4 px-4 sm:px-6 lg:px-10">
        <Link href="/" className="touch-target relative flex items-center gap-2.5 transition-opacity hover:opacity-90">
          <Image
            src="/logo-mark.png"
            alt=""
            width={28}
            height={28}
            className="size-7"
            priority
          />
          <span className="text-[15px] font-semibold tracking-[-0.02em]">Pytxo</span>
        </Link>

        <NavigationMenu className="hidden md:flex">
          <NavigationMenuList className="gap-0.5">
            {desktopLinks.map((link) => (
              <NavigationMenuItem key={link.href}>
                <NavigationMenuLink asChild>
                  <Link
                    href={link.href}
                    className={cn("aperture-link px-3 py-1.5 text-[13px] font-medium text-muted-foreground transition-colors hover:text-foreground")}
                  >
                    {link.label}
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>

        <div className="col-start-3 flex items-center justify-end gap-2">
          <HeaderAuth />
          <Button size="sm" className="hidden h-8 rounded-[4px] bg-white px-3.5 text-black hover:bg-white/85 sm:inline-flex" asChild>
            <Link href="/download">Download</Link>
          </Button>

          <Sheet>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" className="touch-target relative md:hidden">
                <MenuIcon />
                <span className="sr-only">Open menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-72 border-border bg-background">
              <SheetHeader>
                <SheetTitle>Menu</SheetTitle>
              </SheetHeader>
              <nav className="mt-6 flex flex-col gap-1">
                {NAV_LINKS.map((link) => (
                  <SheetClose key={link.href} asChild>
                    <Link
                      href={link.href}
                      className="rounded-md px-3 py-3 text-sm font-medium transition-colors hover:bg-card"
                    >
                      {link.label}
                    </Link>
                  </SheetClose>
                ))}

              </nav>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
