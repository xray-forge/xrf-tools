import { isTauri } from "@tauri-apps/api/core";
import { systemCommands } from "@xrf/ipc/commands/system";
import { Logger } from "@xrf/logging";
import { useEffect, useRef } from "react";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Asks the backend once to show the window when the page's first screen is drawn, as the window is built hidden and
 * would stand empty until then. The backend reveals each window once, so a page reload changes nothing.
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
