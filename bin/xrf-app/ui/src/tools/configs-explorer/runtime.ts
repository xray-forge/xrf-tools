import { ContainerDefinition } from "@/lib/container/container-definition";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

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

export { ConfigsExplorerTool as Component } from "./ConfigsExplorerTool";
