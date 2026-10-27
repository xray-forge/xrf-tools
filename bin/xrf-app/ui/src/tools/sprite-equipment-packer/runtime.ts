import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentPackerTool as Component } from "./SpriteEquipmentPackerTool";

export const container: ContainerDefinition = { bindings: [SpriteEquipmentPackerService] };
