import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { IContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentPackerTool as Component } from "./SpriteEquipmentPackerTool";

export const container: IContainerDefinition = { bindings: [SpriteEquipmentPackerService] };
