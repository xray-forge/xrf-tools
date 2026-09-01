import { default as FormatAlignLeftIcon } from "@mui/icons-material/FormatAlignLeft";
import { lazy } from "react";

import { TRANSLATIONS_FORMATTER_HELP } from "@/applications/translations-formatter/help";
import { TranslationsFormatterService } from "@/applications/translations-formatter/services/formatter";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TRANSLATIONS_FORMATTER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./TranslationsFormatterApplication").then((it) => ({ default: it.TranslationsFormatterApplication }))
  ),
  container: { bindings: [TranslationsFormatterService] },
  preload: () => import("./TranslationsFormatterApplication"),
  description: "Check or normalize JSON translation sources",
  group: EApplicationGroupId.TRANSLATIONS,
  help: TRANSLATIONS_FORMATTER_HELP,
  icon: <FormatAlignLeftIcon />,
  id: EApplicationId.TRANSLATIONS_FORMATTER,
  label: "Translations formatter",
  path: "/translations-formatter",
  status: EApplicationStatus.READY,
};
