import { MenuItem, Select, SelectChangeEvent } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { TranslationsVerifyResult } from "@/applications/translations-verifier/components/TranslationsVerifyResult";
import { TranslationsVerifierService } from "@/applications/translations-verifier/services/verifier";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES } from "@/core/translations";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";
import { Nullable } from "@/lib/types/general";

/** Every language at once, which is the run this screen is usually opened to make. */
const LANGUAGE_CHOICES: ReadonlyArray<string> = [ALL_TRANSLATION_LANGUAGES, ...TRANSLATION_LANGUAGES];

export function TranslationsVerifierApplication(): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);
  const verifierService: TranslationsVerifierService = useInjection(TranslationsVerifierService);

  // The run rather than this view's own flag: a check survives the window being reloaded.
  const job: Nullable<IJobState> = verifierService.job;
  const isRunning: boolean = Boolean(job);
  const [language, setLanguage] = useRememberedValue({
    application: EApplicationId.TRANSLATIONS_VERIFIER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: LANGUAGE_CHOICES,
  });

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_VERIFIER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
  });

  const sourcesPath: Nullable<string> = sources.value;

  const onVerify = useCallback(async () => {
    if (!sourcesPath) {
      return;
    }

    await verifierService.verify(sourcesPath, language);
  }, [language, sourcesPath, verifierService]);

  const onCancel = useCallback(() => verifierService.cancel(), [verifierService]);

  const onLanguageChanged = useCallback(
    (event: SelectChangeEvent<string>) => {
      setLanguage(event.target.value);
    },
    [setLanguage]
  );

  // A different tree or language invalidates whatever the previous run reported.
  useEffect(() => {
    verifierService.reset();
  }, [sourcesPath, language, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid}
      title={"Verify translations"}
      description={"Checks every JSON source for ids a language has no text for. Nothing is written."}
      error={verifierService.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={verifierService.result ? <TranslationsVerifyResult result={verifierService.result} /> : null}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Sources"}
        description={"Translations directory, project, or installation holding the JSON sources"}
        field={sources}
      />

      <FormRow
        label={"Language"}
        description={"One language, or every language the build compiles"}
        controlId={"translations-verifier-language"}
        isInline
      >
        <Select
          id={"translations-verifier-language"}
          size={"small"}
          value={language}
          disabled={isRunning}
          onChange={onLanguageChanged}
        >
          {LANGUAGE_CHOICES.map((it) => (
            <MenuItem key={it} value={it}>
              {it}
            </MenuItem>
          ))}
        </Select>
      </FormRow>
    </PickerForm>
  );
}
