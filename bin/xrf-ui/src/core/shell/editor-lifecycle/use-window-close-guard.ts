import { CloseRequestedEvent, Window } from "@tauri-apps/api/window";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { useEffect } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import { Logger } from "@/lib/logging";
import { resolveAppWindow } from "@/lib/tauri/app-window";

/** Asks about unsaved or running work before the window closes, however the close was asked for. */
export function useWindowCloseGuard(): void {
  const lifecycle: EditorLifecycleService = useInjection(EditorLifecycleService);

  useEffect(() => {
    const appWindow: Nullable<Window> = resolveAppWindow();

    if (!appWindow) {
      return;
    }

    // Held apart from the nullable handle, so the close below needs no check of its own.
    const target: Window = appWindow;

    let isActive: boolean = true;
    let unlisten: Nullable<() => void> = null;

    // Approved first, so a refused approval leaves the window guarded and the next close asks again. Listening still,
    // the page would be asked once more and hold its own close, so it stops just before closing.
    function exit(): void {
      systemCommands
        .approveWindowClose(target.label)
        .then(() => {
          unlisten?.();
          unlisten = null;

          return target.close();
        })
        .catch(Logger.error);
    }

    appWindow
      .onCloseRequested((event: CloseRequestedEvent) => {
        event.preventDefault();
        lifecycle.requestExit(exit);
      })
      .then((it: () => void) => {
        if (isActive) {
          unlisten = it;
        } else {
          it();
        }
      })
      .catch(Logger.error);

    return () => {
      isActive = false;
      unlisten?.();
    };
  }, [lifecycle]);
}
