import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

import { EquipmentSpriteEditor, SpriteEquipmentOpenForm } from "./components/editor";

/** Picker until a sprite is open, editor once it is. */
export function SpriteEquipmentEditorTool(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  return (
    <SessionTool
      root={spriteEquipmentService.sprite}
      isOpen={Boolean(spriteEquipmentService.spriteImage.value)}
      openForm={<SpriteEquipmentOpenForm />}
      workspace={<EquipmentSpriteEditor />}
    />
  );
}
