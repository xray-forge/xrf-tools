import { FormatAlignLeftIcon } from "@xrf/ui/icons/format-align-left";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TRANSLATIONS_FORMATTER_HELP } from "./help";

export const TRANSLATIONS_FORMATTER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Check or normalize JSON translation sources",
    group: EToolGroupId.TRANSLATIONS,
    help: TRANSLATIONS_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EToolId.TRANSLATIONS_FORMATTER,
    label: "Translations formatter",
    path: "/translations-formatter",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
