import { Nullable } from "@xrf/types";
import { RefCallback, useLayoutEffect, useState } from "react";

/** The measured content box of an element, in css pixels. */
export interface IElementSize {
  width: number;
  height: number;
}

/**
 * Tracks how large an element is.
 *
 * @returns A ref to put on the element, and its size, or null until it has been measured.
 */
export function useElementSize<T extends HTMLElement>(): [RefCallback<T>, Nullable<IElementSize>] {
  const [element, setElement] = useState<Nullable<T>>(null);
  const [size, setSize] = useState<Nullable<IElementSize>>(null);

  useLayoutEffect(() => {
    if (!element) {
      return;
    }

    const observed: T = element;

    function measure(): void {
      const next: IElementSize = { width: observed.clientWidth, height: observed.clientHeight };

      if (!next.width || !next.height) {
        return;
      }

      // Compared rather than stored blindly, because a `ResizeObserver` fires for changes in either dimension and for
      // the observation itself, and a fresh object every time would re-render everything reading the size.
      setSize((current: Nullable<IElementSize>) =>
        current && current.width === next.width && current.height === next.height ? current : next
      );
    }

    measure();

    const observer: ResizeObserver = new ResizeObserver(measure);

    observer.observe(observed);

    return () => observer.disconnect();
  }, [element]);

  return [setElement, size];
}
