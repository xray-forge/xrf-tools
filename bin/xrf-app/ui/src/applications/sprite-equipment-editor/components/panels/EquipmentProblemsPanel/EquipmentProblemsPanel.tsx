import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/grid";
import { EditorProblemsPanel, IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { toEquipmentProblems } from "./equipment-problems";

/**
 * Everything worth saying about the open sheet that is not an occupant.
 */
export function EquipmentProblemsPanel({
  "data-testid": dataTestId = "equipment-problems-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);

  const findings: Array<IEditorProblem> = toEquipmentProblems(
    spriteEquipmentService.spriteImage.value?.metadata ?? null,
    gridService.layout
  );

  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      findings={findings}
      emptyDescription={"The sheet was read, its configuration resolved, and every icon fits on it."}
    />
  );
}
