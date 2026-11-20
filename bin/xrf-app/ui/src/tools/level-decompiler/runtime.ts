import { ContainerDefinition } from "@/lib/container/container-definition";
import { LevelDecompilerService } from "@/tools/level-decompiler/services/level-decompiler.service";

export { LevelDecompilerTool as Component } from "./LevelDecompilerTool";

export const container: ContainerDefinition = { bindings: [LevelDecompilerService] };
