import { default as TextureIcon } from "@mui/icons-material/Texture";
import { lazy } from "react";

import { TEXTURES_EXPLORER_HELP } from "@/applications/textures-explorer/help";
import { TextureSurfaceService } from "@/applications/textures-explorer/services/surface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { AssetService } from "@/core/assets/services";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TEXTURES_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./TexturesExplorerApplication").then((it) => ({ default: it.TexturesExplorerApplication }))
  ),
  container: {
    // `AssetService` owns the object URL the decoded texture is shown through, so the preview cannot resolve without
    // it; every surface that shows bytes it decoded binds it the same way.
    bindings: [AssetService, TexturesService, TextureSurfaceService],
  },
  preload: () => import("./TexturesExplorerApplication"),
  description: "Browse textures and what their descriptors declare",
  group: EApplicationGroupId.TEXTURES,
  help: TEXTURES_EXPLORER_HELP,
  icon: <TextureIcon />,
  id: EApplicationId.TEXTURES_EXPLORER,
  label: "Textures explorer",
  path: "/textures-explorer",
  status: EApplicationStatus.READY,
};
