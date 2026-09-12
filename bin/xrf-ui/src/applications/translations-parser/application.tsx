import { default as ImportExportIcon } from "@mui/icons-material/ImportExport";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TRANSLATIONS_PARSER_HELP } from "./help";

export const TRANSLATIONS_PARSER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Import raw XML string tables into JSON sources",
    group: EApplicationGroupId.TRANSLATIONS,
    help: TRANSLATIONS_PARSER_HELP,
    icon: <ImportExportIcon />,
    id: EApplicationId.TRANSLATIONS_PARSER,
    label: "Translations parser",
    path: "/translations-parser",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
