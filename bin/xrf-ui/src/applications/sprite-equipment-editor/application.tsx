import { default as ImageIcon } from "@mui/icons-material/Image";
import { lazy } from "react";

import { AssetService } from "@/core/assets/services";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { SpriteEquipmentService } from "@/core/sprite-equipment";

export const SPRITE_EQUIPMENT_EDITOR_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [AssetService, SpriteEquipmentService] },
  Component: lazy(() =>
    import("./SpriteEquipmentEditorApplication").then((it) => ({ default: it.SpriteEquipmentEditorApplication }))
  ),
  preload: () => import("./SpriteEquipmentEditorApplication"),
  description: "Inspect and edit the icons of an equipment sprite",
  group: EApplicationGroupId.SPRITES,
  icon: <ImageIcon />,
  id: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
  label: "Sprite equipment editor",
  path: "/sprite-equipment-editor",
  status: EApplicationStatus.READY,
};
