"use client";

import Image from "next/image";
import Link from "next/link";
import { MenuIcon } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { HeaderAuth } from "@/components/header-auth";
import { NAV_LINKS } from "@/lib/site";
import { cn } from "@/lib/utils";

function NavLabel({ label, badge }: { label: string; badge?: string }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      {label}
      {badge ? (
        <Badge
          variant="outline"
          className="h-4 border-border bg-card px-1.5 py-0 text-[10px] font-medium uppercase tracking-wide text-muted-foreground"
        >
          {badge}
        </Badge>
      ) : null}
    </span>
  );
}

export function SiteHeader() {
  const desktopLinks = NAV_LINKS.filter((link) => link.href !== "/download");

  return (
    <header className="sticky top-0 z-50 bg-background">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
        <Link href="/" className="flex items-center gap-2.5 transition-opacity hover:opacity-90">
          <Image
            src="/logo-mark.png"
            alt="Pytxo"
            width={28}
            height={28}
            className="size-7"
            priority
          />
          <span className="text-base font-semibold tracking-tight">Pytxo</span>
        </Link>

        <NavigationMenu className="hidden md:flex">
          <NavigationMenuList className="gap-0.5">
            {desktopLinks.map((link) => (
              <NavigationMenuItem key={link.href}>
                <NavigationMenuLink asChild>
                  <Link
                    href={link.href}
                    className={cn(
                      "rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground",
                      "transition-colors hover:bg-card hover:text-foreground",
                    )}
                  >
                    <NavLabel label={link.label} badge={link.badge} />
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>

        <div className="flex items-center gap-2">
          <HeaderAuth />
          <Button size="sm" className="hidden sm:inline-flex" asChild>
            <Link href="/download">Download</Link>
          </Button>

          <Sheet>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" className="md:hidden">
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
                  <Link
                    key={link.href}
                    href={link.href}
                    className="rounded-md px-3 py-2.5 text-sm font-medium transition-colors hover:bg-card"
                  >
                    <NavLabel label={link.label} badge={link.badge} />
                  </Link>
                ))}
                <Link
                  href="/sign-in"
                  className="rounded-md px-3 py-2.5 text-sm font-medium transition-colors hover:bg-card"
                >
                  Sign in
                </Link>
                <Link
                  href="/sign-up"
                  className="rounded-md px-3 py-2.5 text-sm font-medium transition-colors hover:bg-card"
                >
                  Sign up
                </Link>
              </nav>
            </SheetContent>
          </Sheet>
        </div>
      </div>
      <span className="chroma-ribbon" aria-hidden />
    </header>
  );
}
