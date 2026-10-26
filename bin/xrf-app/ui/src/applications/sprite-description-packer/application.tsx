import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const SPRITE_DESCRIPTION_PACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build a description sprite from individual icons",
    group: EApplicationGroupId.SPRITES,
    icon: <Inventory2Icon />,
    id: EApplicationId.SPRITE_DESCRIPTION_PACKER,
    label: "Sprite description packer",
    path: "/sprite-description-packer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
