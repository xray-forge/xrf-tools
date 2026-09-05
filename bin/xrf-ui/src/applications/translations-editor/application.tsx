import { default as TranslateIcon } from "@mui/icons-material/Translate";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const TRANSLATIONS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse and edit localization tables",
    group: EApplicationGroupId.TRANSLATIONS,
    icon: <TranslateIcon />,
    id: EApplicationId.TRANSLATIONS_EDITOR,
    label: "Translations editor",
    path: "/translations-editor",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
