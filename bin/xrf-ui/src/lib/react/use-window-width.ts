import { useSyncExternalStore } from "react";

/** One resize subscription shared by every caller, rather than one per hook instance. */
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
