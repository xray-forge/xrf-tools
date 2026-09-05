import { ReactElement, ReactNode } from "react";

import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { EditorBusyProvider } from "@/core/shell/EditorBusyContext";
import { EditorDirtyProvider } from "@/core/shell/EditorDirtyContext";
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
        <EditorBusyProvider>
          <EditorDirtyProvider>
            <ApplicationShellFrame>{children}</ApplicationShellFrame>
          </EditorDirtyProvider>
        </EditorBusyProvider>
      </EditorPanelsProvider>
    </EditorStatusProvider>
  );
}
