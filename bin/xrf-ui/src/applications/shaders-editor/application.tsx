import { default as PaletteIcon } from "@mui/icons-material/Palette";

import { SHADERS_EDITOR_HELP } from "@/applications/shaders-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

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
