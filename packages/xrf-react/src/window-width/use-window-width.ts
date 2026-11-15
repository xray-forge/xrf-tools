import { useSyncExternalStore } from "react";

/** Stable across renders; each hook instance owns a resize listener and its cleanup. */
function subscribe(onChange: () => void): () => void {
  window.addEventListener("resize", onChange);

  return () => window.removeEventListener("resize", onChange);
}

function getSnapshot(): number {
  return window.innerWidth;
}

/**
 * Tracks the viewport width.
 *
 * @returns Current `window.innerWidth`, re-read on every resize.
 */
export function useWindowWidth(): number {
  return useSyncExternalStore(subscribe, getSnapshot);
}
