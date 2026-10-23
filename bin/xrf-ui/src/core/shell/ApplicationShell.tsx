import { ReactElement, ReactNode } from "react";

import { useSessionChanges } from "@/core/ipc/services/changes";
import { KeybindsDispatcher } from "@/core/keybinds";
import { SettingsDialog } from "@/core/settings/components/SettingsDialog";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { EditorLeaveDialog, useWindowCloseGuard } from "./editor-lifecycle";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame, the root lifecycle service's leave prompt, settings, and the keyboard dispatcher, asks
 * before the window closes, and subscribes the window to committed changes.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  useWindowCloseGuard();
  useSessionChanges();

  return (
    <>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
      <EditorLeaveDialog />
      <SettingsDialog />
      <KeybindsDispatcher />
    </>
  );
}
