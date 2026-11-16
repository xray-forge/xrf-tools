import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

import { SpriteEquipmentEditorOpenForm } from "./components/SpriteEquipmentEditorOpenForm";
import { SpriteEquipmentEditorWorkspace } from "./components/SpriteEquipmentEditorWorkspace";

/** Picker until a sprite is open, editor once it is. */
export function SpriteEquipmentEditorTool(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  return (
    <SessionTool
      root={spriteEquipmentService.sprite}
      isOpen={Boolean(spriteEquipmentService.spriteImage.value)}
      openForm={<SpriteEquipmentEditorOpenForm />}
      workspace={<SpriteEquipmentEditorWorkspace />}
    />
  );
}
