import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { AssetService } from "@/core/assets/services";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { ContainerDefinition } from "@/lib/container/container-definition";

export const container: ContainerDefinition = {
  // No catalog service: this tool opens one file and never lists a root. `AssetService` owns the object URL the
  // decoded texture is shown through, so the preview cannot resolve without it.
  bindings: [AssetService, TextureSelectionService, TextureSurfaceService, TextureEditorService],
};

export { TexturesEditorApplication as Component } from "./TexturesEditorApplication";
