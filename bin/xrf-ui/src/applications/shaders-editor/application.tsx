import { default as PaletteIcon } from "@mui/icons-material/Palette";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SHADERS_EDITOR_HELP } from "./help";

export const SHADERS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit render and level-compiler shader libraries",
    group: EApplicationGroupId.SHADERS,
    help: SHADERS_EDITOR_HELP,
    icon: <PaletteIcon />,
    id: EApplicationId.SHADERS_EDITOR,
    label: "Shaders editor",
    path: "/shaders-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
