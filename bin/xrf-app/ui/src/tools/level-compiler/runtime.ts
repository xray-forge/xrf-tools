import { IContainerDefinition } from "@/lib/container/container-definition";
import { LevelCompilerService } from "@/tools/level-compiler/services/level-compiler.service";

export { LevelCompilerTool as Component } from "./LevelCompilerTool";

export const container: IContainerDefinition = { bindings: [LevelCompilerService] };
