import { default as TextureIcon } from "@mui/icons-material/Texture";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TEXTURES_EXPLORER_HELP } from "./help";

export const TEXTURES_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse textures and what their descriptors declare",
    group: EApplicationGroupId.TEXTURES,
    help: TEXTURES_EXPLORER_HELP,
    icon: <TextureIcon />,
    id: EApplicationId.TEXTURES_EXPLORER,
    label: "Textures explorer",
    path: "/textures-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
