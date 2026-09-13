import { ReactElement, ReactNode } from "react";

import { KeybindsDispatcher } from "@/core/keybinds";

import { ApplicationShellFrame } from "./ApplicationShellFrame";
import { EditorLeaveDialog } from "./editor-lifecycle";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame, the root lifecycle service's leave prompt, and the keyboard dispatcher.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  return (
    <>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
      <EditorLeaveDialog />
      <KeybindsDispatcher />
    </>
  );
}
