import { ContainerDefinition } from "@/lib/container/container-definition";
import { SpawnFileService } from "@/tools/spawn-editor/services/spawn-file.service";

export { SpawnEditorTool as Component } from "./SpawnEditorTool";

export const container: ContainerDefinition = { bindings: [SpawnFileService] };
