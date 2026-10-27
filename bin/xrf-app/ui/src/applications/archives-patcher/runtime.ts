import { ArchivesPatcherService } from "@/applications/archives-patcher/services/archives-patcher.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesPatcherApplication as Component } from "./ArchivesPatcherApplication";

export const container: ContainerDefinition = { bindings: [ArchivesPatcherService] };
