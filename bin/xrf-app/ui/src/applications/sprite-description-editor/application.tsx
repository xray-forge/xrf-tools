import { default as DescriptionIcon } from "@mui/icons-material/Description";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const SPRITE_DESCRIPTION_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Inspect and edit the icons of a description sprite",
    group: EApplicationGroupId.SPRITES,
    icon: <DescriptionIcon />,
    id: EApplicationId.SPRITE_DESCRIPTION_EDITOR,
    label: "Sprite description editor",
    path: "/sprite-description-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
