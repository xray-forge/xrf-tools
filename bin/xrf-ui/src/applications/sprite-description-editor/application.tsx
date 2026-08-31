import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const SPRITE_DESCRIPTION_EDITOR_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./SpriteDescriptionEditorApplication").then((it) => ({ default: it.SpriteDescriptionEditorApplication }))
  ),
  preload: () => import("./SpriteDescriptionEditorApplication"),
  description: "Inspect and edit the icons of a description sprite",
  group: EApplicationGroupId.SPRITES,
  icon: <DescriptionIcon />,
  id: EApplicationId.SPRITE_DESCRIPTION_EDITOR,
  label: "Sprite description editor",
  path: "/sprite-description-editor",
  status: EApplicationStatus.PLANNED,
};
