import { TextureSurfaceService } from "@/applications/textures-explorer/services/surface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { AssetService } from "@/core/assets/services";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TexturesExplorerApplication as Component } from "./TexturesExplorerApplication";

export const container: ContainerDefinition = {
  // `AssetService` owns the object URL the decoded texture is shown through, so the preview cannot resolve without
  // it; every surface that shows bytes it decoded binds it the same way.
  bindings: [AssetService, TexturesService, TextureSurfaceService],
};
