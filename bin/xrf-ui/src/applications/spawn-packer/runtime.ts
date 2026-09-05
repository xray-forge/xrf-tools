import { SpawnFileService } from "@/core/spawn/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { SpawnPackerApplication as Component } from "./SpawnPackerApplication";

export const container: ContainerDefinition = { bindings: [SpawnFileService] };
