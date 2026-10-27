import { SurroundSoundIcon } from "@xrf/ui/icons/surround-sound";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SOUND_ENVIRONMENTS_EDITOR_HELP } from "./help";

export const SOUND_ENVIRONMENTS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit acoustic environments and reverb presets",
    group: EToolGroupId.SOUNDS,
    help: SOUND_ENVIRONMENTS_EDITOR_HELP,
    icon: <SurroundSoundIcon />,
    id: EToolId.SOUND_ENVIRONMENTS_EDITOR,
    label: "Sound environments editor",
    path: "/sound-environments-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
