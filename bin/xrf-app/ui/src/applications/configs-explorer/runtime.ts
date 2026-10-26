import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { ContainerDefinition } from "@/lib/container/container-definition";

export const container: ContainerDefinition = {
  // The project owns which tree is open and what it holds, the document owns whichever config of it is on screen, and
  // the other three own what that config's entry point comes to: its sections, what is wrong with it, and what judges
  // the section the reader selected.
  bindings: [
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsResolvedService,
    ConfigsFindingsService,
    ConfigsSchemeService,
  ],
};

export { ConfigsExplorerApplication as Component } from "./ConfigsExplorerApplication";
