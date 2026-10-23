import { ReactElement, ReactNode } from "react";

import { useSessionChanges } from "@/core/ipc/services/changes";
import { KeybindsDispatcher } from "@/core/keybinds";
import { SettingsDialog } from "@/core/settings/components/SettingsDialog";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { EditorLeaveDialog, useWindowCloseGuard } from "./editor-lifecycle";
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
      <EditorLeaveDialog />
      <SettingsDialog />
      <KeybindsDispatcher />
    </>
  );
}
