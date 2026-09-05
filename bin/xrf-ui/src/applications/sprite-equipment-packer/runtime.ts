import { SpriteEquipmentPackerService } from "@/core/sprite-equipment";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpriteEquipmentPackerApplication as Component } from "./SpriteEquipmentPackerApplication";

export const container: ContainerDefinition = { bindings: [SpriteEquipmentPackerService] };
