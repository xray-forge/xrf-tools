import { isTauri } from "@tauri-apps/api/core";
import { systemCommands } from "@xrf/ipc/commands/system";
import { Logger } from "@xrf/logging";
import { useEffect, useRef } from "react";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Asks the backend, once, to put the window on screen when the page's first screen has been drawn. The window is built
 * hidden, and shown any earlier it stands empty until the webview draws into it. The backend reveals each window once:
 * a page reloaded over a shown window changes nothing.
 */
export function useWindowReveal(): void {
  const isAsked = useRef<boolean>(false);

  useEffect(() => {
    if (isAsked.current || !isTauri()) {
      return;
    }

    isAsked.current = true;

    // The frame after the commit lays the screen out and paints it; the one after that follows the paint.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        systemCommands.revealWindow().catch((error: unknown) => {
          log.error("Failed to reveal the window, which shows once the backend stops waiting:", error);
        });
      });
    });
  }, []);
}
