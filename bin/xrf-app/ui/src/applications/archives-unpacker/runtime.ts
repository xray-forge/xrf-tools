import { UnpackerService } from "@/applications/archives-unpacker/services/unpacker";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesUnpackerApplication as Component } from "./ArchivesUnpackerApplication";

export const container: ContainerDefinition = { bindings: [UnpackerService] };
