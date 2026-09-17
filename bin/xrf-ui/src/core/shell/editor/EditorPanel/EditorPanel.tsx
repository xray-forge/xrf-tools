import { ReactElement, ReactNode } from "react";

import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelProps extends BaseComponentProps {
  /** The name this panel's stripe button carries. */
  title: string;
  /** Controls acting on the whole panel, at the end of the title row. */
  actions?: ReactNode;
  children: ReactNode;
}

/**
 * One panel, titled with the name its stripe button carries.
 */
export function EditorPanel({
  "data-testid": dataTestId = "editor-panel",
  id,
  className,
  title,
  actions,
  children,
}: IEditorPanelProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 min-w-0 flex-col", className)}>
      <EditorPanelHeader title={title} actions={actions} />

      <div className={"min-h-0 min-w-0 grow overflow-y-auto"}>{children}</div>
    </div>
  );
}
