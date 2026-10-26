import { default as SurroundSoundIcon } from "@mui/icons-material/SurroundSound";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SOUND_ENVIRONMENTS_EDITOR_HELP } from "./help";

export const SOUND_ENVIRONMENTS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit acoustic environments and reverb presets",
    group: EApplicationGroupId.SOUNDS,
    help: SOUND_ENVIRONMENTS_EDITOR_HELP,
    icon: <SurroundSoundIcon />,
    id: EApplicationId.SOUND_ENVIRONMENTS_EDITOR,
    label: "Sound environments editor",
    path: "/sound-environments-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
