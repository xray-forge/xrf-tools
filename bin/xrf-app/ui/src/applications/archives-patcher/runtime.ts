import { PatcherService } from "@/applications/archives-patcher/services/patcher";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesPatcherApplication as Component } from "./ArchivesPatcherApplication";

export const container: ContainerDefinition = { bindings: [PatcherService] };
