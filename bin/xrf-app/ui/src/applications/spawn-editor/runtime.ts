import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnEditorApplication as Component } from "./SpawnEditorApplication";

export const container: ContainerDefinition = { bindings: [SpawnFileService] };
