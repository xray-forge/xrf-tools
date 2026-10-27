import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/equipment-grid";
import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/sprite-equipment-editor";
import { AssetService } from "@/core/assets/services";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentEditorApplication as Component } from "./SpriteEquipmentEditorApplication";

export const container: ContainerDefinition = {
  bindings: [AssetService, SpriteEquipmentPackerService, SpriteEquipmentEditorService, EquipmentGridService],
};
