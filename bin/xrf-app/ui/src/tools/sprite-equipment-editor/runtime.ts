import { AssetService } from "@/core/assets/services";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { EquipmentGridService } from "@/tools/sprite-equipment-editor/services/equipment-grid";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

export { SpriteEquipmentEditorTool as Component } from "./SpriteEquipmentEditorTool";

export const container: ContainerDefinition = {
  bindings: [AssetService, SpriteEquipmentPackerService, SpriteEquipmentEditorService, EquipmentGridService],
};
