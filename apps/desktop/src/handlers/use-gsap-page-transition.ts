import { useLayoutEffect, useRef } from "react";
import { gsap } from "gsap";

export function useGsapPageTransition(dependency: string, enabled = true) {
  const elementRef = useRef<HTMLDivElement | null>(null);

  useLayoutEffect(() => {
    const element = elementRef.current;
    if (!element || !enabled || prefersReducedMotion()) return;

    const context = gsap.context(() => {
      gsap.fromTo(
        element,
        {
          autoAlpha: 0,
          scale: 0.985,
          y: 22,
        },
        {
          autoAlpha: 1,
          duration: 0.58,
          scale: 1,
          y: 0,
        }
      );
    }, element);

    return () => context.revert();
  }, [dependency, enabled]);

  return elementRef;
}

function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}