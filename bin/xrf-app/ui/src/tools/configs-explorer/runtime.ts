import { ContainerDefinition } from "@/lib/container/container-definition";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindService } from "@/tools/configs-explorer/services/configs-find";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

export const container: ContainerDefinition = {
  // The project owns which tree is open and what it holds, the document owns whichever config of it is on screen, three
  // own what that config's entry point comes to - its sections, what is wrong with it, and what judges the section the
  // reader selected - and the last owns where the whole project writes what a reader searched for.
  bindings: [
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsResolvedService,
    ConfigsFindingsService,
    ConfigsSchemeService,
    ConfigsFindService,
  ],
};

export { ConfigsExplorerTool as Component } from "./ConfigsExplorerTool";
