import { Container } from "@wirestate/core";

import { AssetService } from "@/core/assets/services";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services";
import { VisualRenderService } from "@/core/visuals/services/visual-render";
import { VisualViewService } from "@/core/visuals/services/visual-view";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";
import { ArchivesExplorerKeybindsService } from "@/tools/archives-explorer/services/archives-explorer-keybinds";

export { ArchivesExplorerTool as Component } from "./ArchivesExplorerTool";

export const container: ContainerDefinition = {
  bindings: [
    AssetService,
    ArchivesService,
    ArchivePreviewService,
    ArchivesExplorerKeybindsService,
    VisualLoadService,
    VisualViewService,
    VisualRenderService,
    // Nothing here poses a model or marks a bone: the loader alone says what the model preview draws.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(VisualLoadService) },
  ],
};
