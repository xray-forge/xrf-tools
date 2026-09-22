import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { Nullable } from "@xrf/types";
import { useCallback, useEffect, useState } from "react";

export interface IWindowControls {
  /** False whenever the window cannot be driven, so the caller can render the bar without controls. */
  isAvailable: boolean;
  isMaximized: boolean;
  minimize: () => void;
  toggleMaximize: () => void;
  close: () => void;
}

/**
 * Resolves the window that contains this document.
 *
 * Reads the handle lazily rather than at module scope: the accessor throws outside a tauri webview, and
 * the frontend is also served by `vite preview` and rendered under jsdom by the tests.
 *
 * @returns The current Tauri window, or `null` outside a Tauri webview.
 */
function resolveAppWindow(): Nullable<Window> {
  if (!isTauri()) {
    return null;
  }

  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

/**
 * Provides controls for a custom host-window title bar.
 *
 * The maximized flag is tracked rather than asked for on every render because the window can be
 * maximized without the bar being touched - a double click on the drag region, a snap gesture, or the
 * keyboard - and the button glyph has to follow all of them.
 *
 * @returns The current window state and control callbacks.
 */
export function useWindowControls(): IWindowControls {
  const [appWindow] = useState<Nullable<Window>>(resolveAppWindow);
  const [isMaximized, setMaximized] = useState<boolean>(false);

  const minimize = useCallback(() => {
    appWindow?.minimize().catch(console.error);
  }, [appWindow]);

  const toggleMaximize = useCallback(() => {
    appWindow?.toggleMaximize().catch(console.error);
  }, [appWindow]);

  const close = useCallback(() => {
    appWindow?.close().catch(console.error);
  }, [appWindow]);

  useEffect(() => {
    if (!appWindow) {
      return;
    }

    let isActive: boolean = true;

    function sync(): void {
      appWindow
        ?.isMaximized()
        .then((next: boolean) => {
          if (isActive) {
            setMaximized(next);
          }
        })
        .catch(console.error);
    }

    sync();

    // Resize covers every path into and out of the maximized state, including the ones that never
    // reach this component: snap layouts, the taskbar menu, and Win+Up.
    const unlisten: Promise<() => void> = appWindow.onResized(sync);

    return () => {
      isActive = false;
      unlisten.then((it: () => void) => it()).catch(console.error);
    };
  }, [appWindow]);

  return { isAvailable: appWindow !== null, isMaximized, minimize, toggleMaximize, close };
}
