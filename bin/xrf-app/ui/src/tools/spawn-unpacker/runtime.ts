import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnUnpackerTool as Component } from "./SpawnUnpackerTool";

export const container: ContainerDefinition = { bindings: [SpawnConversionService] };
