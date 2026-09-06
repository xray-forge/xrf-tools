import { RefCallback, useCallback, useLayoutEffect, useState } from "react";

import { Nullable } from "@/lib/types/general";

/** The measured content box of an element, in css pixels. */
export interface IElementSize {
  width: number;
  height: number;
}

/**
 * Tracks how large an element is.
 *
 * Measured in a layout effect and again on every resize, so the first render a caller lays anything out on is already
 * the right one and no frame is composed against a stale box.
 *
 * A box reporting nothing - collapsed, detached, or a panel someone closed - leaves the last real measurement in place
 * rather than replacing it with zero: that keeps a size the caller can still lay out against, and the element comes
 * back the size it went away at.
 *
 * @returns A ref to put on the element, and its size, or null until it has been measured.
 */
export function useElementSize<T extends HTMLElement>(): [RefCallback<T>, Nullable<IElementSize>] {
  const [element, setElement] = useState<Nullable<T>>(null);
  const [size, setSize] = useState<Nullable<IElementSize>>(null);

  const ref = useCallback<RefCallback<T>>((next: Nullable<T>): void => setElement(next), []);

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

  return [ref, size];
}
