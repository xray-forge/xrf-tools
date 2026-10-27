import { BuildIcon } from "@xrf/ui/icons/build";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TRANSLATIONS_BUILDER_HELP } from "./help";

export const TRANSLATIONS_BUILDER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build per-language string tables from translation sources",
    group: EToolGroupId.TRANSLATIONS,
    help: TRANSLATIONS_BUILDER_HELP,
    icon: <BuildIcon />,
    id: EToolId.TRANSLATIONS_BUILDER,
    label: "Translations builder",
    path: "/translations-builder",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
