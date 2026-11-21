import { IContainerDefinition } from "@/lib/container/container-definition";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindService } from "@/tools/configs-explorer/services/configs-find";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

export const container: IContainerDefinition = {
  // The open project, the config on screen, three views of that config, and the project-wide find.
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
