import { AssetService } from "@/core/assets/services";
import { TextureRenderService } from "@/core/textures/services/texture-render";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { TextureSurfaceService } from "@/core/textures/services/texture-surface";
import { TextureViewService } from "@/core/textures/services/texture-view";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { TextureCatalogService } from "@/tools/textures-explorer/services/texture-catalog.service";

export const container: ContainerDefinition = {
  // The catalog owns the browsed roots and the selection owns whichever texture the tree chose from them.
  // `AssetService` owns the object URL the decoded texture is shown through, so the preview cannot resolve without it.
  bindings: [
    AssetService,
    TextureSelectionService,
    TextureCatalogService,
    TextureSurfaceService,
    TextureViewService,
    TextureRenderService,
  ],
};

export { TexturesExplorerTool as Component } from "./TexturesExplorerTool";
