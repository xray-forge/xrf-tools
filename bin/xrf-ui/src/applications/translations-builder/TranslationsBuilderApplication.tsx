import { Alert, MenuItem, Select, SelectChangeEvent, Switch } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsBuildResult } from "@/applications/translations-builder/components/TranslationsBuildResult";
import { createRoots } from "@/core/assets/lib";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationBuildSummary } from "@/core/bindings/types/xrf-app";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { getProjectGamedataPath, getProjectTranslationsPath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES } from "@/core/translations";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Every language at once, which is the ordinary build. */
const LANGUAGE_CHOICES: ReadonlyArray<string> = [ALL_TRANSLATION_LANGUAGES, ...TRANSLATION_LANGUAGES];

export function TranslationsBuilderApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const projectService: ProjectService = useInjection(ProjectService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<TranslationBuildSummary>>(null);
  const [isSorted, setIsSorted] = useState<boolean>(true);
  const [language, setLanguage] = useRememberedValue({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: LANGUAGE_CHOICES,
  });

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getProjectTranslationsPath(projectService.xrfProjectPath) : null,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isLoading,
    seed: async () => (projectService.xrfProjectPath ? getProjectGamedataPath(projectService.xrfProjectPath) : null),
  });

  const sourcesPath: Nullable<string> = sources.value;
  const outputPath: Nullable<string> = destination.value;

  const onBuild = useCallback(async () => {
    if (!sourcesPath || !outputPath) {
      return;
    }

    try {
      setIsLoading(true);
      setResult(null);
      setError(null);

      log.info("Building translations:", sourcesPath, language, "sorted:", isSorted);

      const built: TranslationBuildSummary = await translationsCommands.buildProject(
        createRoots([sourcesPath]),
        null,
        language,
        outputPath,
        isSorted
      );

      setResult(built);

      notify({
        details: `${sourcesPath}\n${outputPath}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.TRANSLATIONS_BUILDER,
        title: `Built ${built.files} string table(s)`,
      });
    } catch (caught: unknown) {
      log.error("Build error:", caught);
      setError(String(caught));

      notify({
        details: `${sourcesPath}\n${String(caught)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.TRANSLATIONS_BUILDER,
        title: "Could not build translations",
      });
    } finally {
      setIsLoading(false);
    }
  }, [sourcesPath, outputPath, language, isSorted, log, notify]);

  const onLanguageChanged = useCallback(
    (event: SelectChangeEvent<string>) => {
      setLanguage(event.target.value);
    },
    [setLanguage]
  );

  // Anything the build depends on invalidates whatever the previous run reported.
  useEffect(() => {
    setResult(null);
    setError(null);
  }, [sourcesPath, outputPath, language, isSorted]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!sources.isValid || !destination.isValid}
      title={"Build translations"}
      description={"Compiles JSON sources into one X-Ray string table per language, in each language's code page."}
      error={error ?? undefined}
      submitLabel={"Build"}
      status={result ? <Alert severity={"success"}>Translations built.</Alert> : null}
      result={result ? <TranslationsBuildResult result={result} /> : null}
      onSubmit={onBuild}
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
        controlId={"translations-builder-language"}
        isInline
      >
        <Select
          id={"translations-builder-language"}
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

      <PathFormRow
        isDisabled={isLoading}
        label={"Output"}
        description={"Directory the string tables are written to, as <output>/<language>/<name>.xml"}
        field={destination}
      />

      <FormRow
        label={"Sort ids"}
        description={"Off preserves the order each source declares them in"}
        controlId={"translations-builder-sort"}
        isRequired={false}
        isInline
      >
        <Switch
          id={"translations-builder-sort"}
          size={"small"}
          checked={isSorted}
          disabled={isLoading}
          onChange={(event) => setIsSorted(event.target.checked)}
        />
      </FormRow>
    </PickerForm>
  );
}
