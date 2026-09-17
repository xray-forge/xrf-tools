import { ReactElement } from "react";

import { EquipmentSpriteViewer } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentSpriteViewer";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
