import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TRANSLATIONS_VERIFIER_HELP } from "./help";

export const TRANSLATIONS_VERIFIER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Report translations missing from one or more languages",
    group: EApplicationGroupId.TRANSLATIONS,
    help: TRANSLATIONS_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EApplicationId.TRANSLATIONS_VERIFIER,
    label: "Translations verifier",
    path: "/translations-verifier",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
