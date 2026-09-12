import { default as BuildIcon } from "@mui/icons-material/Build";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TRANSLATIONS_BUILDER_HELP } from "./help";

export const TRANSLATIONS_BUILDER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build per-language string tables from translation sources",
    group: EApplicationGroupId.TRANSLATIONS,
    help: TRANSLATIONS_BUILDER_HELP,
    icon: <BuildIcon />,
    id: EApplicationId.TRANSLATIONS_BUILDER,
    label: "Translations builder",
    path: "/translations-builder",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
