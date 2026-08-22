import type { HTMLAttributes, ReactElement } from "react";

import { cn } from "@/lib/utils";
import { Badge } from "./badge";
import { Button } from "./button";
import { X } from "lucide-react";

export interface ToastProps extends HTMLAttributes<HTMLDivElement> {
  title: string;
  detail?: string;
  tone?: "success" | "warning" | "error" | "neutral";
  onDismiss?: () => void;
}

export function Toast({ className, title, detail, tone, onDismiss, ...props }: ToastProps): ReactElement {
  return (
    <div
      className={cn(
        "flex items-start justify-between gap-3 rounded-none border bg-card p-4 shadow-lg",
        tone === "error" && "border-destructive",
        tone === "warning" && "border-primary",
        tone === "success" && "border-green-500",
        tone === "neutral" && "border-border",
        className
      )}
      {...props}
    >
      <div className="grid gap-1">
        <span className="text-sm font-semibold text-foreground">{title}</span>
        {detail && <span className="text-sm text-muted-foreground">{detail}</span>}
      </div>
      {onDismiss && (
        <Button
          aria-label={`Dismiss ${title}`}
          type="button"
          variant="ghost"
          size="icon"
          onClick={onDismiss}
        >
          <X size={14} />
        </Button>
      )}
    </div>
  );
}

export { Toaster } from "./sonner";



