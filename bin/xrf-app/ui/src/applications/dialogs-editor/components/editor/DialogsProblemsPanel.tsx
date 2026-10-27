import { DialogFinding } from "@xrf/ipc/types/xrf-dialog";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorProblemsPanel } from "@/core/shell/editor/EditorProblemsPanel";

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
