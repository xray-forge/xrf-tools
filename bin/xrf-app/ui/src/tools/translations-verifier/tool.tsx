import { FactCheckIcon } from "@xrf/ui/icons/fact-check";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TRANSLATIONS_VERIFIER_HELP } from "./help";

export const TRANSLATIONS_VERIFIER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Report translations missing from one or more languages",
    group: EToolGroupId.TRANSLATIONS,
    help: TRANSLATIONS_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EToolId.TRANSLATIONS_VERIFIER,
    label: "Translations verifier",
    path: "/translations-verifier",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
