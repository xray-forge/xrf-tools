import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { EquipmentSpriteViewer } from "@/tools/sprite-equipment-editor/components/sprite-view/EquipmentSpriteViewer";

export function EquipmentSpriteEditorWorkspace({
  "data-testid": dataTestId = "equipment-sprite-editor-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("workspace flex max-h-full max-w-full grow items-center justify-center", className)}
    >
      <EquipmentSpriteViewer />
    </div>
  );
}
