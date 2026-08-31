import { default as ImportExportIcon } from "@mui/icons-material/ImportExport";
import { lazy } from "react";

import { TRANSLATIONS_PARSER_HELP } from "@/applications/translations-parser/help";
import { TranslationsParserService } from "@/applications/translations-parser/services/parser";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TRANSLATIONS_PARSER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./TranslationsParserApplication").then((it) => ({ default: it.TranslationsParserApplication }))
  ),
  container: { bindings: [TranslationsParserService] },
  preload: () => import("./TranslationsParserApplication"),
  description: "Import raw XML string tables into JSON sources",
  group: EApplicationGroupId.TRANSLATIONS,
  help: TRANSLATIONS_PARSER_HELP,
  icon: <ImportExportIcon />,
  id: EApplicationId.TRANSLATIONS_PARSER,
  label: "Translations parser",
  path: "/translations-parser",
  status: EApplicationStatus.READY,
};
