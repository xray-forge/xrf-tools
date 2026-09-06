import { AssetService } from "@/core/assets/services";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { ContainerDefinition } from "@/lib/container/container-definition";

export const container: ContainerDefinition = {
  // The catalog owns the browsed roots and the selection owns whichever texture the tree chose from them.
  // `AssetService` owns the object URL the decoded texture is shown through, so the preview cannot resolve without it.
  bindings: [AssetService, TextureSelectionService, TextureCatalogService, TextureSurfaceService],
};

export { TexturesExplorerApplication as Component } from "./TexturesExplorerApplication";
