import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnUnpackerApplication as Component } from "./SpawnUnpackerApplication";

export const container: ContainerDefinition = { bindings: [SpawnConversionService] };
