import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { lazy } from "react";

import { AssetService } from "@/core/assets/services";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { SpriteEquipmentService } from "@/core/sprite-equipment";

export const SPRITE_EQUIPMENT_PACKER_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [AssetService, SpriteEquipmentService] },
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
