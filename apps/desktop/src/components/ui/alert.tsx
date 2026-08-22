import type { HTMLAttributes, ReactElement } from "react";

import { cn } from "@/lib/utils";
import { Badge } from "./badge";

export interface AlertProps extends HTMLAttributes<HTMLDivElement> {
  tone?: "neutral" | "brand" | "success" | "warning" | "danger";
}

export function Alert({
  children,
  className,
  tone = "neutral",
  ...props
}: AlertProps): ReactElement {
  return (
    <div
      role="alert"
      className={cn(
        "grid gap-2 rounded-none border bg-card p-4 shadow-sm",
        tone === "danger" && "border-destructive",
        tone === "warning" && "border-primary",
        tone === "success" && "border-green-500",
        tone === "brand" && "border-primary",
        tone === "neutral" && "border-border",
        className
      )}
      {...props}
    >
      <Badge tone={tone}>{tone}</Badge>
      <div className="text-sm text-foreground">{children}</div>
    </div>
  );
}



