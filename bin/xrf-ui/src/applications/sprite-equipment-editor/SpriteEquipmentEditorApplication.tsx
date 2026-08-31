import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { EquipmentSpriteEditor } from "@/applications/sprite-equipment-editor/components/equipment-editor/EquipmentSpriteEditor";
import { SpriteEquipmentOpenForm } from "@/applications/sprite-equipment-editor/components/equipment-editor/SpriteEquipmentOpenForm";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { SpriteEquipmentService } from "@/core/sprite-equipment";

/** Picker until a sprite is open, editor once it is. */
export function SpriteEquipmentEditorApplication(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentService = useInjection(SpriteEquipmentService);

  if (spriteEquipmentService.isReady) {
    return spriteEquipmentService.spriteImage.value ? <EquipmentSpriteEditor /> : <SpriteEquipmentOpenForm />;
  }

  return <ApplicationLoader />;
}
