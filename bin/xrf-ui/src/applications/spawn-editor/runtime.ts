import { SpawnFileService } from "@/core/spawn/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnEditorApplication as Component } from "./SpawnEditorApplication";

export const container: ContainerDefinition = { bindings: [SpawnFileService] };
