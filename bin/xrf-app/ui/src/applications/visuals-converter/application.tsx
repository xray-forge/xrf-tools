import { default as SwapHorizIcon } from "@mui/icons-material/SwapHoriz";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { VISUALS_CONVERTER_HELP } from "./help";

export const VISUALS_CONVERTER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Convert models and animations between game and editable formats",
    group: EApplicationGroupId.VISUALS,
    help: VISUALS_CONVERTER_HELP,
    icon: <SwapHorizIcon />,
    id: EApplicationId.VISUALS_CONVERTER,
    label: "Visuals converter",
    path: "/visuals-converter",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
