import type { ComponentProps, ReactElement } from "react";

import { cn } from "@/lib/utils";

export type SidebarProps = ComponentProps<"aside">;

export function Sidebar({ className, ...props }: SidebarProps): ReactElement {
  return (
    <aside
      className={cn("flex min-h-0 flex-col rounded-none bg-sidebar", className)}
      {...props}
    />
  );
}



