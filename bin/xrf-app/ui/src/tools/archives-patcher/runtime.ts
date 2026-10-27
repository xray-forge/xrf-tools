import { ContainerDefinition } from "@/lib/container/container-definition";
import { ArchivesPatcherService } from "@/tools/archives-patcher/services/archives-patcher.service";

export { ArchivesPatcherTool as Component } from "./ArchivesPatcherTool";

export const container: ContainerDefinition = { bindings: [ArchivesPatcherService] };
