import { useInjection } from "@wirestate/react";
import { ETranslationLanguage } from "@xrf/ipc/types/xrf-translation";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { TranslationsVerifierService } from "@/tools/translations-verifier/services/translations-verifier.service";

import { TranslationsVerifyResult } from "./components/TranslationsVerifyResult";

interface ITranslationsVerifyRequest {
  sources: string;
  /** One language, or every language. */
  language: ETranslationLanguage;
}

export function TranslationsVerifierTool(): ReactElement {
  const verifierService: TranslationsVerifierService = useInjection(TranslationsVerifierService);

  // The run rather than this view's own flag: a check survives the window being reloaded.
  const isRunning: boolean = verifierService.operation.isRunning;
  const [language, setLanguage] = useRememberedValue({
    tool: EToolId.TRANSLATIONS_VERIFIER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: TRANSLATION_LANGUAGES_WITH_ALL,
  });

  const sources: IPathField = usePathField({
    tool: EToolId.TRANSLATIONS_VERIFIER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const request: Nullable<ITranslationsVerifyRequest> = useMemo(
    () => (sources.value ? { sources: sources.value, language } : null),
    [language, sources.value]
  );

  const onVerify = useCallback(
    async (submitted: ITranslationsVerifyRequest) => {
      await verifierService.verify(submitted.sources, submitted.language);
    },
    [verifierService]
  );

  return (
    <JobPickerForm
      operation={verifierService.operation}
      request={request}
      isSubmitDisabled={!sources.isValid}
      title={"Verify translations"}
      description={"Checks every JSON source for ids a language has no text for. Nothing is written."}
      submitLabel={"Verify"}
      renderResult={(result) => <TranslationsVerifyResult result={result} />}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Sources"}
        description={"Translations directory, project, or installation holding the JSON sources"}
        field={sources}
      />

      <TranslationLanguageField
        id={"translations-verifier-language"}
        description={"One language, or every language to check"}
        value={language}
        isAllAllowed
        isDisabled={isRunning}
        onChange={setLanguage}
      />
    </JobPickerForm>
  );
}
