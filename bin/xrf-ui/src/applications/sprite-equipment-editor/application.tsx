import { default as ImageIcon } from "@mui/icons-material/Image";
import { lazy } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { AssetService } from "@/core/assets/services";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";

export const SPRITE_EQUIPMENT_EDITOR_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [AssetService, SpriteEquipmentPackerService, SpriteEquipmentEditorService] },
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
