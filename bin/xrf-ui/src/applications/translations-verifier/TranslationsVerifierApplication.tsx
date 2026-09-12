import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { TranslationsVerifierService } from "@/applications/translations-verifier/services/verifier";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

import { TranslationsVerifyResult } from "./components/TranslationsVerifyResult";

export function TranslationsVerifierApplication(): ReactElement {
  const verifierService: TranslationsVerifierService = useInjection(TranslationsVerifierService);

  // The run rather than this view's own flag: a check survives the window being reloaded.
  const job: Nullable<IJobState> = verifierService.operation.job;
  const isRunning: boolean = verifierService.operation.isRunning;
  const [language, setLanguage] = useRememberedValue({
    application: EApplicationId.TRANSLATIONS_VERIFIER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: TRANSLATION_LANGUAGES_WITH_ALL,
  });

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_VERIFIER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const sourcesPath: Nullable<string> = sources.value;

  const onVerify = useCallback(async () => {
    if (!sourcesPath) {
      return;
    }

    await verifierService.verify(sourcesPath, language);
  }, [language, sourcesPath, verifierService]);

  const onCancel = useCallback(() => verifierService.operation.cancel(), [verifierService]);

  // A different tree or language invalidates whatever the previous run reported.
  useEffect(() => {
    verifierService.operation.reset();
  }, [sourcesPath, language, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid}
      title={"Verify translations"}
      description={"Checks every JSON source for ids a language has no text for. Nothing is written."}
      error={verifierService.operation.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        verifierService.operation.result ? <TranslationsVerifyResult result={verifierService.operation.result} /> : null
      }
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
    </PickerForm>
  );
}
