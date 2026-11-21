import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { IContainerDefinition } from "@/lib/container/container-definition";

export { SpawnUnpackerTool as Component } from "./SpawnUnpackerTool";

export const container: IContainerDefinition = { bindings: [SpawnConversionService] };
