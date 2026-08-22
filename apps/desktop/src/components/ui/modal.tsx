import * as React from "react";
import type { ReactElement, ReactNode } from "react";
import { X } from "lucide-react";

import { cn } from "@/lib/utils";

export interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  children?: React.ReactNode;
  footer?: React.ReactNode;
}

export function Modal(props: ModalProps): ReactElement | null {
  React.useEffect(() => {
    if (!props.open) return;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") props.onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [props.open, props.onClose]);

  if (!props.open) return null;

  return (
    <div
      className="fixed inset-0 z-[90] grid place-items-center bg-background/70 p-4 backdrop-blur-xl"
      onClick={props.onClose}
      role="presentation"
    >
      <div
        className={cn("grid max-h-[85vh] w-full max-w-lg overflow-hidden rounded-none border bg-card shadow-2xl")}
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label={props.title}
      >
        <header className="flex items-start justify-between gap-3 border-b border-border p-5">
          <div className="grid gap-1">
            <h2 className="m-0 text-lg font-semibold leading-6 text-foreground">{props.title}</h2>
            {props.subtitle && <p className="m-0 text-sm leading-5 text-muted-foreground">{props.subtitle}</p>}
          </div>
          <button
            type="button"
            aria-label="Close"
            onClick={props.onClose}
            className="grid h-9 w-9 place-items-center rounded-none text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <X size={18} />
          </button>
        </header>
        <div className="overflow-auto p-5">{props.children}</div>
        {props.footer && (
          <footer className="flex flex-wrap justify-end gap-2 border-t border-border bg-muted/40 p-4">{props.footer}</footer>
        )}
      </div>
    </div>
  );
}
