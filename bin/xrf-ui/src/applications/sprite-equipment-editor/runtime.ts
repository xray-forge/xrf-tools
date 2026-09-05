import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { AssetService } from "@/core/assets/services";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentEditorApplication as Component } from "./SpriteEquipmentEditorApplication";

export const container: ContainerDefinition = {
  bindings: [AssetService, SpriteEquipmentPackerService, SpriteEquipmentEditorService],
};
