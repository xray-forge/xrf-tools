import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { Nullable } from "@xrf/types";

/**
 * Resolves the window that contains this document.
 *
 * Reads the handle lazily rather than at module scope: the accessor throws outside a tauri webview, and
 * the frontend is also served by `vite preview` and rendered under jsdom by the tests.
 *
 * @returns The current Tauri window, or `null` outside a Tauri webview.
 */
export function resolveAppWindow(): Nullable<Window> {
  if (!isTauri()) {
    return null;
  }

  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}
