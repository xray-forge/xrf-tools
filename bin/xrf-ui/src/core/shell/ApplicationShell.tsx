import { ReactElement, ReactNode } from "react";

import { KeybindsDispatcher } from "@/core/keybinds";
import { SettingsDialog } from "@/core/settings/components/SettingsDialog";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { EditorLeaveDialog, useWindowCloseGuard } from "./editor-lifecycle";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame, the root lifecycle service's leave prompt, settings, and the keyboard dispatcher, and asks
 * before the window closes.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  useWindowCloseGuard();

  return (
    <>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
      <EditorLeaveDialog />
      <SettingsDialog />
      <KeybindsDispatcher />
    </>
  );
}
