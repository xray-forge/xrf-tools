import { ContainerDefinition } from "@/lib/container/container-definition";
import { ArchivesUnpackerService } from "@/tools/archives-unpacker/services/archives-unpacker.service";

export { ArchivesUnpackerTool as Component } from "./ArchivesUnpackerTool";

export const container: ContainerDefinition = { bindings: [ArchivesUnpackerService] };
