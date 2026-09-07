import { ReactElement, ReactNode } from "react";

import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { EditorLeaveDialog } from "@/core/shell/editor-lifecycle";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Renders the shell frame and the root lifecycle service's leave prompt.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  return (
    <>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
      <EditorLeaveDialog />
    </>
  );
}
