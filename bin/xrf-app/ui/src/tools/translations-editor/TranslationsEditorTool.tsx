import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";

import { TranslationsEditor } from "./components/TranslationsEditor";
import { TranslationsEditorOpenForm } from "./components/TranslationsEditorOpenForm";
import { TranslationsService } from "./services/translations.service";

/** Picker until a project is open, editor once it is. */
export function TranslationsEditorTool(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  return (
    <SessionTool
      root={translationsService.project}
      openForm={<TranslationsEditorOpenForm />}
      workspace={<TranslationsEditor />}
    />
  );
}
