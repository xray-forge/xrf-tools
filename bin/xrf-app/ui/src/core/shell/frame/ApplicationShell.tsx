import { useSessionChanges } from "@xrf/ipc/react";
import { ReactElement, ReactNode } from "react";

import { KeybindsDispatcher } from "@/core/keybinds";
import { SettingsDialog } from "@/core/settings-dialog/components/SettingsDialog";
import { ToolLeaveDialog, useWindowCloseGuard } from "@/core/shell/lifecycle";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { useRetainedSessionScopes } from "./use-retained-session-scopes";
import { useWindowReveal } from "./use-window-reveal";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame, the root lifecycle service's leave prompt, settings and the keyboard dispatcher. Asks before
 * the window closes, subscribes it to committed changes, tells the backend which tools the page can open, and shows
 * the window once this first screen is drawn.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  useWindowReveal();
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
