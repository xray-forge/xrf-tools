import { IContainerDefinition } from "@/lib/container/container-definition";
import { ArchivesPackerService } from "@/tools/archives-packer/services/archives-packer.service";

export { ArchivesPackerTool as Component } from "./ArchivesPackerTool";

export const container: IContainerDefinition = { bindings: [ArchivesPackerService] };
