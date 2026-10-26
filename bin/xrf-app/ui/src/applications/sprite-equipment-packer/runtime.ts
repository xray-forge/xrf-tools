import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentPackerApplication as Component } from "./SpriteEquipmentPackerApplication";

export const container: ContainerDefinition = { bindings: [SpriteEquipmentPackerService] };
