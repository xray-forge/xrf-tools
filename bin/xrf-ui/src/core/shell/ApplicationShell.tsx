import { ReactElement, ReactNode } from "react";

import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { EditorLeaveDialog } from "@/core/shell/editor-lifecycle";
import { EditorStatusProvider } from "@/core/shell/EditorStatusContext";
import { EditorPanelsProvider } from "@/core/shell/panel/context";

interface IApplicationShellProps {
  children: ReactNode;
}

/**
 * Supplies the shell's contexts and renders the frame that consumes them.
 *
 * Split so the frame can read those contexts: a provider cannot consume what it provides.
 */
export function ApplicationShell({ children }: IApplicationShellProps): ReactElement {
  return (
    <EditorStatusProvider>
      <EditorPanelsProvider>
        <ApplicationShellFrame>{children}</ApplicationShellFrame>

        <EditorLeaveDialog />
      </EditorPanelsProvider>
    </EditorStatusProvider>
  );
}
