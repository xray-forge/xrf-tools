import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/chrome/ToolLoader";

import { TranslationsEditor } from "./components/TranslationsEditor";
import { TranslationsEditorOpenForm } from "./components/TranslationsEditorOpenForm";
import { TranslationsService } from "./services/translations.service";

/** Picker until a project is open, editor once it is. */
export function TranslationsEditorTool(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  if (translationsService.isReady) {
    return translationsService.project.value ? <TranslationsEditor /> : <TranslationsEditorOpenForm />;
  }

  return <ToolLoader />;
}
