import { default as PeopleIcon } from "@mui/icons-material/People";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const CHARACTERS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./CharactersExplorerApplication").then((it) => ({ default: it.CharactersExplorerApplication }))
  ),
  preload: () => import("./CharactersExplorerApplication"),
  description: "Browse character profiles and the assets they reference",
  group: EApplicationGroupId.GAMEPLAY,
  icon: <PeopleIcon />,
  id: EApplicationId.CHARACTERS_EXPLORER,
  label: "Characters explorer",
  path: "/characters-explorer",
  status: EApplicationStatus.PLANNED,
};
