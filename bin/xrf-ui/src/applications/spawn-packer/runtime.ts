import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnPackerApplication as Component } from "./SpawnPackerApplication";

export const container: ContainerDefinition = { bindings: [SpawnConversionService] };
