import { cn } from "@/lib/utils";

type IconProps = {
  className?: string;
  title?: string;
};

export function WindowsIcon({ className, title = "Windows" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden={title ? undefined : true}
      role={title ? "img" : undefined}
      className={cn("size-6", className)}
    >
      {title ? <title>{title}</title> : null}
      <path d="M3 5.5 10.5 4.4v7.1H3V5.5Zm0 8.1h7.5v7.1L3 19.6v-5.9ZM11.7 4.2 21 3v8.5h-9.3V4.2Zm0 9.3H21V21l-9.3-1.3v-6.2Z" />
    </svg>
  );
}

export function AppleIcon({ className, title = "macOS" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden={title ? undefined : true}
      role={title ? "img" : undefined}
      className={cn("size-6", className)}
    >
      {title ? <title>{title}</title> : null}
      <path d="M16.7 12.7c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3.1-1.6-1.3-.1-2.5.8-3.2.8-.7 0-1.7-.8-2.8-.7-1.4 0-2.8.9-3.5 2.2-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.6 2.2 2.8 2.1 1.1 0 1.5-.7 2.9-.7 1.3 0 1.7.7 2.9.7 1.2 0 1.9-1 2.6-2 .8-1.2 1.1-2.3 1.1-2.4-.1 0-2.2-.8-2.2-3.8ZM14.8 6.4c.6-.8 1.1-1.8.9-2.9-1 .1-2.1.7-2.7 1.5-.6.7-1.1 1.8-.9 2.8 1.1.1 2.1-.5 2.7-1.4Z" />
    </svg>
  );
}

export function LinuxIcon({ className, title = "Linux" }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden={title ? undefined : true}
      role={title ? "img" : undefined}
      className={cn("size-6", className)}
    >
      {title ? <title>{title}</title> : null}
      <path d="M12.1 2.2c-.7 0-1.4.5-1.8 1.3-.5 1.1-.4 2.5.1 4.1-.7.2-1.4.6-1.9 1.2-.8 1-1.1 2.4-.8 3.8.2 1.1.8 2.1 1.6 2.8-.2.7-.4 1.5-.4 2.3 0 1.4.5 2.7 1.4 3.6.7.7 1.6 1.1 2.6 1.1.5 0 1-.1 1.4-.3.4.2.9.3 1.4.3 1 0 1.9-.4 2.6-1.1.9-.9 1.4-2.2 1.4-3.6 0-.8-.2-1.6-.4-2.3.8-.7 1.4-1.7 1.6-2.8.3-1.4 0-2.8-.8-3.8-.5-.6-1.2-1-1.9-1.2.5-1.6.6-3 .1-4.1-.4-.8-1.1-1.3-1.8-1.3h-.9Zm-.5 1.6h.9c.2 0 .4.2.5.4.3.7.2 1.7-.1 2.8h-1.6c-.3-1.1-.4-2.1-.1-2.8.1-.2.3-.4.5-.4Zm-2.2 5.1c.3 0 .6.1.9.3l.6 2.1c-.6.3-1 .9-1.1 1.6-.2-.2-.3-.5-.3-.8 0-1.1.4-2.1 1.1-2.8l-.1-.1c-.3-.2-.7-.3-1.1-.3Zm5.2 0c-.4 0-.8.1-1.1.3l-.1.1c.7.7 1.1 1.7 1.1 2.8 0 .3-.1.6-.3.8-.1-.7-.5-1.3-1.1-1.6l.6-2.1c.3-.2.6-.3.9-.3Zm-2.6 1.2h.1l-.7 2.4h1.3l-.7-2.4Zm-1.4 3.3h2.8c.4.5.6 1.1.6 1.8 0 .9-.3 1.7-.9 2.3-.5.5-1.1.7-1.8.7s-1.3-.2-1.8-.7c-.6-.6-.9-1.4-.9-2.3 0-.7.2-1.3.6-1.8Z" />
    </svg>
  );
}

export type DetectedOs = "windows" | "macos" | "linux" | "unknown";

export function detectOs(): DetectedOs {
  if (typeof navigator === "undefined") return "unknown";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux") || ua.includes("x11")) return "linux";
  return "unknown";
}
