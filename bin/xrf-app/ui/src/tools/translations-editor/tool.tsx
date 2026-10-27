import { TranslateIcon } from "@xrf/ui/icons/translate";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TRANSLATIONS_EDITOR_HELP } from "./help";

export const TRANSLATIONS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse and edit localization tables",
    group: EToolGroupId.TRANSLATIONS,
    icon: <TranslateIcon />,
    help: TRANSLATIONS_EDITOR_HELP,
    id: EToolId.TRANSLATIONS_EDITOR,
    label: "Translations editor",
    path: "/translations-editor",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
