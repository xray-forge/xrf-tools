import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IContainerDefinition } from "@/lib/container/container-definition";

export { SpawnPackerTool as Component } from "./SpawnPackerTool";

export const container: IContainerDefinition = { bindings: [SpawnConversionService] };
