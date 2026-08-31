import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";
import { lazy } from "react";

import { TRANSLATIONS_VERIFIER_HELP } from "@/applications/translations-verifier/help";
import { TranslationsVerifierService } from "@/applications/translations-verifier/services/verifier";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TRANSLATIONS_VERIFIER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./TranslationsVerifierApplication").then((it) => ({ default: it.TranslationsVerifierApplication }))
  ),
  container: { bindings: [TranslationsVerifierService] },
  preload: () => import("./TranslationsVerifierApplication"),
  description: "Report translations missing from one or more languages",
  group: EApplicationGroupId.TRANSLATIONS,
  help: TRANSLATIONS_VERIFIER_HELP,
  icon: <FactCheckIcon />,
  id: EApplicationId.TRANSLATIONS_VERIFIER,
  label: "Translations verifier",
  path: "/translations-verifier",
  status: EApplicationStatus.READY,
};
