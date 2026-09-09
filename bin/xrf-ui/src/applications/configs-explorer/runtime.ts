import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ContainerDefinition } from "@/lib/container/container-definition";

export const container: ContainerDefinition = {
  // The project owns which tree is open and what it holds; the document owns whichever config of it is on screen.
  bindings: [ConfigsProjectService, ConfigsDocumentService],
};

export { ConfigsExplorerApplication as Component } from "./ConfigsExplorerApplication";
