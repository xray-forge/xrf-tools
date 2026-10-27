import { useSessionChanges } from "@xrf/ipc/react";
import { ReactElement, ReactNode } from "react";

import { KeybindsDispatcher } from "@/core/keybinds";
import { SettingsDialog } from "@/core/settings/components/SettingsDialog";
import { ToolLeaveDialog, useWindowCloseGuard } from "@/core/shell/lifecycle";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { useRetainedSessionScopes } from "./use-retained-session-scopes";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame, the root lifecycle service's leave prompt, settings, and the keyboard dispatcher, asks
 * before the window closes, subscribes the window to committed changes, and tells the backend which tools the page
 * holds openings for.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  useWindowCloseGuard();
  useSessionChanges();
  useRetainedSessionScopes();

  return (
    <>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
      <ToolLeaveDialog />
      <SettingsDialog />
      <KeybindsDispatcher />
    </>
  );
}
