import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TranslationsEditor } from "@/applications/translations-editor/components/TranslationsEditor";
import { TranslationsEditorOpenForm } from "@/applications/translations-editor/components/TranslationsEditorOpenForm";
import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

/** Picker until a project is open, editor once it is. */
export function TranslationsEditorApplication(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  if (translationsService.isReady) {
    return translationsService.project.value ? <TranslationsEditor /> : <TranslationsEditorOpenForm />;
  }

  return <ApplicationLoader />;
}
