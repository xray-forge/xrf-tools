import { PackerService } from "@/applications/archives-packer/services/packer";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesPackerApplication as Component } from "./ArchivesPackerApplication";

export const container: ContainerDefinition = { bindings: [PackerService] };
