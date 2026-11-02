import { ImportExportIcon } from "@xrf/ui/icons/import-export";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TRANSLATIONS_PARSER_HELP } from "./help";

export const TRANSLATIONS_PARSER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Import raw XML string tables into JSON sources",
    group: EToolGroupId.TRANSLATIONS,
    help: TRANSLATIONS_PARSER_HELP,
    icon: <ImportExportIcon />,
    id: EToolId.TRANSLATIONS_PARSER,
    label: "Translations parser",
    path: "/translations-parser",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
