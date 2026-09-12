import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

import { TranslationsEditor } from "./components/TranslationsEditor";
import { TranslationsEditorOpenForm } from "./components/TranslationsEditorOpenForm";
import { TranslationsService } from "./services/translations";

/** Picker until a project is open, editor once it is. */
export function TranslationsEditorApplication(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  if (translationsService.isReady) {
    return translationsService.project.value ? <TranslationsEditor /> : <TranslationsEditorOpenForm />;
  }

  return <ApplicationLoader />;
}
