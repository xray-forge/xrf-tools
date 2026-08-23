import { default as TranslateIcon } from "@mui/icons-material/Translate";
import { lazy } from "react";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TRANSLATIONS_EDITOR_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [TranslationsService] },
  Component: lazy(() =>
    import("./TranslationsEditorApplication").then((it) => ({ default: it.TranslationsEditorApplication }))
  ),
  preload: () => import("./TranslationsEditorApplication"),
  description: "Browse and edit localization tables",
  group: EApplicationGroupId.TRANSLATIONS,
  icon: <TranslateIcon />,
  id: EApplicationId.TRANSLATIONS_EDITOR,
  label: "Translations editor",
  path: "/translations-editor",
  status: EApplicationStatus.READY,
};
