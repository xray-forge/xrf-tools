import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const SPRITE_EQUIPMENT_UNPACKER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./SpriteEquipmentUnpackerApplication").then((it) => ({ default: it.SpriteEquipmentUnpackerApplication }))
  ),
  preload: () => import("./SpriteEquipmentUnpackerApplication"),
  description: "Extract individual icons from an equipment sprite",
  group: EApplicationGroupId.SPRITES,
  icon: <UnarchiveIcon />,
  id: EApplicationId.SPRITE_EQUIPMENT_UNPACKER,
  label: "Sprite equipment unpacker",
  path: "/sprite-equipment-unpacker",
  status: EApplicationStatus.PLANNED,
};
