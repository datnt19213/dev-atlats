import type { HTMLAttributes, ReactElement } from "react";

import { cn } from "@/lib/utils";

export function Typography({
  children,
  className,
  variant = "body",
  ...props
}: HTMLAttributes<HTMLParagraphElement> & {
  variant?: "h1" | "h2" | "h3" | "body" | "muted" | "caption";
}): ReactElement {
  const Component =
    variant === "h1"
      ? "h1"
      : variant === "h2"
        ? "h2"
        : variant === "h3"
          ? "h3"
          : "p";

  return (
    <Component
      {...props}
      className={cn(
        "m-0 tracking-tight",
        variant === "h1" && "text-3xl font-bold leading-tight",
        variant === "h2" && "text-2xl font-semibold leading-tight",
        variant === "h3" && "text-lg font-medium leading-7",
        variant === "body" && "text-base leading-7",
        variant === "muted" && "text-sm leading-6 text-muted-foreground",
        variant === "caption" && "text-xs leading-5 text-muted-foreground",
        className
      )}
    >
      {children}
    </Component>
  );
}



