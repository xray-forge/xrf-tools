import { ReactElement } from "react";

import { DialogFinding } from "@/core/ipc/types/xrf-dialog";
import { EditorProblemsPanel } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IDialogsProblemsPanelProps extends BaseComponentProps {
  findings: ReadonlyArray<DialogFinding>;
}

/**
 * Problems reported while reading the application's project.
 */
export function DialogsProblemsPanel({
  "data-testid": dataTestId = "dialogs-editor-problems-panel",
  id,
  className,
  findings,
}: IDialogsProblemsPanelProps): ReactElement {
  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      findings={findings}
      rulePrefix={"dialog."}
      emptyDescription={"Every dialog and string table in this project read cleanly."}
    />
  );
}
