import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment";

export const SPRITE_EQUIPMENT_PACKER_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [SpriteEquipmentPackerService] },
  Component: lazy(() =>
    import("./SpriteEquipmentPackerApplication").then((it) => ({ default: it.SpriteEquipmentPackerApplication }))
  ),
  preload: () => import("./SpriteEquipmentPackerApplication"),
  description: "Build an equipment sprite from individual icons",
  group: EApplicationGroupId.SPRITES,
  icon: <Inventory2Icon />,
  id: EApplicationId.SPRITE_EQUIPMENT_PACKER,
  label: "Sprite equipment packer",
  path: "/sprite-equipment-packer",
  status: EApplicationStatus.READY,
};
