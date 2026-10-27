import { AssetService } from "@/core/assets/services";
import { TextureRenderService } from "@/core/textures/services/texture-render";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { TextureSurfaceService } from "@/core/textures/services/texture-surface";
import { TextureViewService } from "@/core/textures/services/texture-view";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { TextureBumpService } from "@/tools/textures-editor/services/texture-bump";
import { TextureEditorService } from "@/tools/textures-editor/services/texture-editor";
import { TextureEncodingService } from "@/tools/textures-editor/services/texture-encoding";

export const container: ContainerDefinition = {
  // No catalog service: this tool opens one file and never lists a root. `AssetService` owns the object URL the
  // decoded texture is shown through, so the preview cannot resolve without it.
  bindings: [
    AssetService,
    TextureSelectionService,
    TextureSurfaceService,
    TextureViewService,
    TextureRenderService,
    TextureEncodingService,
    TextureEditorService,
    TextureBumpService,
  ],
};

export { TexturesEditorTool as Component } from "./TexturesEditorTool";
