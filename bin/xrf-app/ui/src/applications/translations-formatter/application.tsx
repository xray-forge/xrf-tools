import { default as FormatAlignLeftIcon } from "@mui/icons-material/FormatAlignLeft";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TRANSLATIONS_FORMATTER_HELP } from "./help";

export const TRANSLATIONS_FORMATTER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Check or normalize JSON translation sources",
    group: EApplicationGroupId.TRANSLATIONS,
    help: TRANSLATIONS_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EApplicationId.TRANSLATIONS_FORMATTER,
    label: "Translations formatter",
    path: "/translations-formatter",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
