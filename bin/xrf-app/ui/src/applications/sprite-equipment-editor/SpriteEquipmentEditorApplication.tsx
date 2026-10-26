import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

import { EquipmentSpriteEditor, SpriteEquipmentOpenForm } from "./components/editor";

/** Picker until a sprite is open, editor once it is. */
export function SpriteEquipmentEditorApplication(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  if (spriteEquipmentService.isReady) {
    return spriteEquipmentService.spriteImage.value ? <EquipmentSpriteEditor /> : <SpriteEquipmentOpenForm />;
  }

  return <ApplicationLoader />;
}
