import type { HTMLAttributes, ReactElement, ReactNode } from "react";

import { Label } from "./label";
import { cn } from "@/lib/utils";

export function Field({
  children,
  className,
  description,
  label,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  description?: ReactNode;
  label?: ReactNode;
}): ReactElement {
  return (
    <div className={cn("grid gap-2.5", className)} {...props}>
      {label ? <Label>{label}</Label> : null}
      {children}
      {description ? (
        <p className="text-sm text-muted-foreground">{description}</p>
      ) : null}
    </div>
  );
}



