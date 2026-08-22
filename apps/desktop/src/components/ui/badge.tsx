import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center rounded-none border px-2.5 py-0.5 text-xs font-semibold transition-colors focus-visible:ring-1 focus-visible:ring-ring",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive:
          "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
        outline: "text-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {
  tone?: "neutral" | "brand" | "success" | "warning" | "danger";
}

function Badge({ className, variant, tone, ...props }: BadgeProps) {
  const resolvedVariant = tone === "danger" || tone === "warning"
    ? (tone === "danger" ? "destructive" : "default")
    : tone === "brand"
      ? "default"
      : tone === "success"
        ? "secondary"
        : "outline";

  return (
    <div className={cn(badgeVariants({ variant: resolvedVariant }), className)} {...props} />
  )
}

export { Badge, badgeVariants }




