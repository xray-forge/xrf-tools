import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const SPRITE_DESCRIPTION_UNPACKER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./SpriteDescriptionUnpackerApplication").then((it) => ({
      default: it.SpriteDescriptionUnpackerApplication,
    }))
  ),
  preload: () => import("./SpriteDescriptionUnpackerApplication"),
  description: "Extract individual icons from a description sprite",
  group: EApplicationGroupId.SPRITES,
  icon: <UnarchiveIcon />,
  id: EApplicationId.SPRITE_DESCRIPTION_UNPACKER,
  label: "Sprite description unpacker",
  path: "/sprite-description-unpacker",
  status: EApplicationStatus.PLANNED,
};
