import { ArchivesPackerService } from "@/applications/archives-packer/services/archives-packer.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesPackerApplication as Component } from "./ArchivesPackerApplication";

export const container: ContainerDefinition = { bindings: [ArchivesPackerService] };
