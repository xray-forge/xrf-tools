import { ContainerDefinition } from "@/lib/container/container-definition";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

export { SpawnEditorTool as Component } from "./SpawnEditorTool";

export const container: ContainerDefinition = { bindings: [SpawnEditorService] };
