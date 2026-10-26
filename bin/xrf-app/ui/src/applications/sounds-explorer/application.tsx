import { default as LibraryMusicIcon } from "@mui/icons-material/LibraryMusic";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SOUNDS_EXPLORER_HELP } from "./help";

export const SOUNDS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Listen to sounds and inspect their X-Ray metadata",
    group: EApplicationGroupId.SOUNDS,
    help: SOUNDS_EXPLORER_HELP,
    icon: <LibraryMusicIcon />,
    id: EApplicationId.SOUNDS_EXPLORER,
    label: "Sounds explorer",
    path: "/sounds-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
