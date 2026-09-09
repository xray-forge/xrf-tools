import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ContainerDefinition } from "@/lib/container/container-definition";

export const container: ContainerDefinition = {
  // The project owns which tree is open and what it holds, the document owns whichever config of it is on screen,
  // and the resolved service owns what that config's entry point comes to.
  bindings: [ConfigsProjectService, ConfigsDocumentService, ConfigsResolvedService],
};

export { ConfigsExplorerApplication as Component } from "./ConfigsExplorerApplication";
