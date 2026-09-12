import { default as ImageIcon } from "@mui/icons-material/Image";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPRITE_EQUIPMENT_EDITOR_HELP } from "./help";

export const SPRITE_EQUIPMENT_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Inspect and edit the icons of an equipment sprite",
    group: EApplicationGroupId.SPRITES,
    icon: <ImageIcon />,
    help: SPRITE_EQUIPMENT_EDITOR_HELP,
    id: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
    label: "Sprite equipment editor",
    path: "/sprite-equipment-editor",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
