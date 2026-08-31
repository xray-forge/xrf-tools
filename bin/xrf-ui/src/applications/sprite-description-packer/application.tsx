import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const SPRITE_DESCRIPTION_PACKER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./SpriteDescriptionPackerApplication").then((it) => ({ default: it.SpriteDescriptionPackerApplication }))
  ),
  preload: () => import("./SpriteDescriptionPackerApplication"),
  description: "Build a description sprite from individual icons",
  group: EApplicationGroupId.SPRITES,
  icon: <Inventory2Icon />,
  id: EApplicationId.SPRITE_DESCRIPTION_PACKER,
  label: "Sprite description packer",
  path: "/sprite-description-packer",
  status: EApplicationStatus.PLANNED,
};
