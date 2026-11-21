import { IContainerDefinition } from "@/lib/container/container-definition";
import { SpawnCompilerService } from "@/tools/spawn-compiler/services/spawn-compiler.service";

export { SpawnCompilerTool as Component } from "./SpawnCompilerTool";

export const container: IContainerDefinition = { bindings: [SpawnCompilerService] };
