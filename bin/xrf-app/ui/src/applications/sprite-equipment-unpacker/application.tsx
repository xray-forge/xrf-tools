import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const SPRITE_EQUIPMENT_UNPACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Extract individual icons from an equipment sprite",
    group: EApplicationGroupId.SPRITES,
    icon: <UnarchiveIcon />,
    id: EApplicationId.SPRITE_EQUIPMENT_UNPACKER,
    label: "Sprite equipment unpacker",
    path: "/sprite-equipment-unpacker",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
