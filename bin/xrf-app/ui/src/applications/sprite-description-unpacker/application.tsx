import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const SPRITE_DESCRIPTION_UNPACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Extract individual icons from a description sprite",
    group: EApplicationGroupId.SPRITES,
    icon: <UnarchiveIcon />,
    id: EApplicationId.SPRITE_DESCRIPTION_UNPACKER,
    label: "Sprite description unpacker",
    path: "/sprite-description-unpacker",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
