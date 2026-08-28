import { Alert, MenuItem, Select, SelectChangeEvent } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsVerifyResult } from "@/applications/translations-verifier/components/TranslationsVerifyResult";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES } from "@/core/translations";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Every language at once, which is the run this screen is usually opened to make. */
const LANGUAGE_CHOICES: ReadonlyArray<string> = [ALL_TRANSLATION_LANGUAGES, ...TRANSLATION_LANGUAGES];

export function TranslationsVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const pathsService: PathsService = useInjection(PathsService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<TranslationVerifySummary>>(null);
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
    isDisabled: isLoading,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
  });

  const sourcesPath: Nullable<string> = sources.value;

  const onVerify = useCallback(async () => {
    if (!sourcesPath) {
      return;
    }

    try {
      setIsLoading(true);
      setResult(null);
      setError(null);

      log.info("Verifying translations:", sourcesPath, language);

      const verified: TranslationVerifySummary = await translationsCommands.verifyProject(
        createRoots([sourcesPath]),
        null,
        language
      );

      setResult(verified);

      notify({
        details: String(sourcesPath),
        severity: verified.missing ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
        source: EApplicationId.TRANSLATIONS_VERIFIER,
        title: verified.missing
          ? `Translations are incomplete: ${verified.missing} missing`
          : "Translations are complete",
      });
    } catch (caught: unknown) {
      log.error("Verify error:", caught);
      setError(String(caught));

      notify({
        details: `${sourcesPath}\n${String(caught)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_VERIFIER,
        title: "Could not check translations",
      });
    } finally {
      setIsLoading(false);
    }
  }, [sourcesPath, language, log, notify]);

  const onLanguageChanged = useCallback(
    (event: SelectChangeEvent<string>) => {
      setLanguage(event.target.value);
    },
    [setLanguage]
  );

  // A different tree or language invalidates whatever the previous run reported.
  useEffect(() => {
    setResult(null);
    setError(null);
  }, [sourcesPath, language]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!sources.isValid}
      title={"Verify translations"}
      description={"Checks every JSON source for ids a language has no text for. Nothing is written."}
      error={error ?? undefined}
      submitLabel={"Verify"}
      status={
        result ? (
          result.missing ? (
            <Alert severity={"warning"}>Some languages are incomplete.</Alert>
          ) : (
            <Alert severity={"success"}>Every language is complete.</Alert>
          )
        ) : null
      }
      result={result ? <TranslationsVerifyResult result={result} /> : null}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isLoading}
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
          disabled={isLoading}
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
