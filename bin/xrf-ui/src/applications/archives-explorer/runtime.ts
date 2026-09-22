import { Container } from "@wirestate/core";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchivesExplorerKeybindsService } from "@/applications/archives-explorer/services/keybinds";
import { AssetService } from "@/core/assets/services";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { ArchivesExplorerApplication as Component } from "./ArchivesExplorerApplication";

export const container: ContainerDefinition = {
  bindings: [
    AssetService,
    ArchivesService,
    ArchivesExplorerKeybindsService,
    VisualLoadService,
    VisualViewService,
    VisualRenderService,
    // Nothing here poses a model or marks a bone: the loader alone says what the model preview draws.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(VisualLoadService) },
  ],
};
