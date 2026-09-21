import { default as TuneIcon } from "@mui/icons-material/Tune";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SOUNDS_EDITOR_HELP } from "./help";

export const SOUNDS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit sound parameters and prepare game audio",
    group: EApplicationGroupId.SOUNDS,
    help: SOUNDS_EDITOR_HELP,
    icon: <TuneIcon />,
    id: EApplicationId.SOUNDS_EDITOR,
    label: "Sounds editor",
    path: "/sounds-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
