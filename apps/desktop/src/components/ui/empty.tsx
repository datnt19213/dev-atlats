import type { ReactElement } from "react";

export function Empty({
  children = "No content available.",
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>): ReactElement {
  return (
    <div
      {...props}
      className="flex min-h-32 flex-col items-center justify-center gap-4 rounded-none bg-muted/50 p-8 text-center"
    >
      {children}
    </div>
  );
}



