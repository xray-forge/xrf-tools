import { default as TranslateIcon } from "@mui/icons-material/Translate";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TRANSLATIONS_EDITOR_HELP } from "./help";

export const TRANSLATIONS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse and edit localization tables",
    group: EApplicationGroupId.TRANSLATIONS,
    icon: <TranslateIcon />,
    help: TRANSLATIONS_EDITOR_HELP,
    id: EApplicationId.TRANSLATIONS_EDITOR,
    label: "Translations editor",
    path: "/translations-editor",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
