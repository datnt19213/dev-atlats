import type { ReactElement } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import mermaid from "mermaid";
import { encode } from "plantuml-encoder";

export interface DiagramPreviewProps {
  content: string;
  format: string;
  className?: string;
}

export function DiagramPreview(props: DiagramPreviewProps): ReactElement {
  const ref = useRef<HTMLDivElement | null>(null);
  const [error, setError] = useState<string | null>(null);
  const normalizedFormat = useMemo(() => props.format.toLowerCase(), [props.format]);

  useEffect(() => {
    if (!ref.current) return;
    setError(null);
    ref.current.innerHTML = "";

    if (normalizedFormat === "mermaid") {
      const id = "mermaid-" + Math.random().toString(36).slice(2);
      mermaid.initialize({ startOnLoad: false, theme: "default" });
      mermaid.render(id, props.content)
        .then(({ svg }) => {
          if (ref.current) ref.current.innerHTML = svg;
        })
        .catch((err) => {
          setError(err instanceof Error ? err.message : "Failed to render diagram");
        });
    } else if (normalizedFormat === "plantuml") {
      try {
        const encoded = encode(props.content);
        const url = "https://www.plantuml.com/plantuml/svg/" + encoded;
        const img = document.createElement("img");
        img.src = url;
        img.style.maxWidth = "100%";
        img.style.height = "auto";
        img.onload = () => { if (ref.current) ref.current.appendChild(img); };
        img.onerror = () => setError("Failed to load PlantUML diagram");
      } catch (err) {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setError(err instanceof Error ? err.message : "Failed to encode PlantUML");
      }
    } else if (normalizedFormat === "svg") {
      if (ref.current) {
        ref.current.innerHTML = props.content;
        const svg = ref.current.querySelector("svg");
        if (svg) {
          svg.style.maxWidth = "100%";
          svg.style.height = "auto";
        }
      }
    } else {
      setError("Unsupported diagram format: " + props.format);
    }
  }, [props.content, props.format, normalizedFormat]);

  if (error) {
    return (
      <div className={props.className}>
        <div className="rounded-none border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive">
          Failed to render diagram: {error}
        </div>
      </div>
    );
  }

  return <div ref={ref} className={props.className} />;
}
