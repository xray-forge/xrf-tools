import { EditorProblemsList, IEditorProblemsListProps } from "@xrf/ui/editor/problems";
import { ReactElement } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";

/**
 * The docked Problems panel: a tool's findings under the panel's heading.
 */
export function EditorProblemsPanel({
  "data-testid": dataTestId = "editor-problems-panel",
  id,
  className,
  ...list
}: IEditorProblemsListProps): ReactElement {
  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Problems"}>
      <EditorProblemsList {...list} />
    </EditorPanel>
  );
}
