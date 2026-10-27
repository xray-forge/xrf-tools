import { ArchivesUnpackerService } from "@/applications/archives-unpacker/services/archives-unpacker.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesUnpackerApplication as Component } from "./ArchivesUnpackerApplication";

export const container: ContainerDefinition = { bindings: [ArchivesUnpackerService] };
