import { CloseRequestedEvent, Window } from "@tauri-apps/api/window";
import { useInjection } from "@wirestate/react";
import { systemCommands } from "@xrf/ipc/commands/system";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { useEffect } from "react";

import { ToolLifecycleService } from "@/core/shell/services/tool-lifecycle";
import { resolveAppWindow } from "@/lib/tauri/app-window";

/** Asks about unsaved or running work before the window closes, however the close was asked for. */
export function useWindowCloseGuard(): void {
  const lifecycle: ToolLifecycleService = useInjection(ToolLifecycleService);
  const log: Logger = useLogger(__MODULE_NAME__);

  useEffect(() => {
    const appWindow: Nullable<Window> = resolveAppWindow();

    if (!appWindow) {
      return;
    }

    // Held apart from the nullable handle, so the close below needs no check of its own.
    const target: Window = appWindow;

    let isActive: boolean = true;
    let unlisten: Nullable<() => void> = null;

    function guard(): void {
      target
        .onCloseRequested((event: CloseRequestedEvent) => {
          log.info("Window close requested, asking first:", target.label);

          event.preventDefault();
          lifecycle.requestExit(exit);
        })
        .then((it: () => void) => {
          if (isActive) {
            log.info("Guarding window close:", target.label);
            unlisten = it;
          } else {
            it();
          }
        })
        .catch((error: unknown) => log.error("Cannot guard window close:", target.label, error));
    }

    // Approved first, so a refused approval leaves the window guarded and the next close asks again. Listening still,
    // the page would be asked about its own close once more, so it stops just before closing, and guards it again where
    // the close then fails. The backend finishes an approved close itself, so the close need not wait for the backend to
    // hear that the page stopped.
    function exit(): void {
      log.info("Approving window close:", target.label);

      systemCommands
        .approveWindowClose()
        .then(() => {
          log.info("Window close approved, closing:", target.label);

          unlisten?.();
          unlisten = null;

          return target.close();
        })
        .catch((error: unknown) => {
          log.error("Window close failed, the window stays guarded:", target.label, error);

          if (isActive && !unlisten) {
            guard();
          }
        });
    }

    guard();

    return () => {
      isActive = false;
      unlisten?.();
    };
  }, [lifecycle, log]);
}
