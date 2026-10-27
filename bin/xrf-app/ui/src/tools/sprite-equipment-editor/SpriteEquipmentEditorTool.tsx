import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

import { EquipmentSpriteEditor, SpriteEquipmentOpenForm } from "./components/editor";

/** Picker until a sprite is open, editor once it is. */
export function SpriteEquipmentEditorTool(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  if (spriteEquipmentService.isReady) {
    return spriteEquipmentService.spriteImage.value ? <EquipmentSpriteEditor /> : <SpriteEquipmentOpenForm />;
  }

  return <ToolLoader />;
}
