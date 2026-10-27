import { isTauri } from "@tauri-apps/api/core";
import { sessionsCommands } from "@xrf/ipc/commands/sessions";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { useEffect, useRef } from "react";

import { useCurrentTool } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Tells the backend once, as the page starts, which of the window's tools are live: the shell's and the tool
 * the page opens on, whose openings it attaches to. A tool the page before it held and this one does not was left
 * without a teardown, the page having reloaded, so the backend closes what it held.
 */
export function useRetainedSessionScopes(): void {
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const isRetained = useRef<boolean>(false);

  useEffect(() => {
    if (isRetained.current || !isTauri()) {
      return;
    }

    isRetained.current = true;

    const live: Array<string> = [SHELL_SESSION_TOOL, ...(tool ? [tool.id] : [])];

    sessionsCommands.retainScopes(live).catch((error: unknown) => {
      log.error("Failed to release what the window's previous page held:", error);
    });
  }, [tool]);
}
