import { Nullable } from "@xrf/types";

/**
 * Tells a listener the display's scale each time it changes, as the window moves to a screen of another scale or the
 * page is zoomed, until stopped.
 *
 * @param listener - Hears the new ratio of device pixels to CSS pixels.
 * @returns What stops it.
 */
export function watchDevicePixelRatio(listener: (ratio: number) => void): () => void {
  if (typeof window.matchMedia !== "function") {
    return () => undefined;
  }

  let query: Nullable<MediaQueryList> = null;

  // A query matches one ratio only, so each change watches again from the ratio it changed to.
  function watch(): void {
    query = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
    query.addEventListener("change", onChange, { once: true });
  }

  function onChange(): void {
    watch();
    listener(window.devicePixelRatio);
  }

  watch();

  return () => query?.removeEventListener("change", onChange);
}
