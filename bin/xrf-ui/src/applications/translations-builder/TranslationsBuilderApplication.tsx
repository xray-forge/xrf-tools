import { MenuItem, Select, SelectChangeEvent, Switch } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsBuildResult } from "@/applications/translations-builder/components/TranslationsBuildResult";
import { TranslationsBuilderService } from "@/applications/translations-builder/services/builder";
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

/** Every language at once, which is the ordinary build. */
const LANGUAGE_CHOICES: ReadonlyArray<string> = [ALL_TRANSLATION_LANGUAGES, ...TRANSLATION_LANGUAGES];

export function TranslationsBuilderApplication(): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);
  const builderService: TranslationsBuilderService = useInjection(TranslationsBuilderService);

  // The run rather than this view's own flag: a build survives the window being reloaded.
  const job: Nullable<IJobState> = builderService.job;
  const isRunning: boolean = Boolean(job);

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
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.GAMEDATA, pathsService.paths),
  });

  const sourcesPath: Nullable<string> = sources.value;
  const outputPath: Nullable<string> = destination.value;

  const onBuild = useCallback(async () => {
    if (!sourcesPath || !outputPath) {
      return;
    }

    await builderService.build(sourcesPath, language, outputPath, isSorted);
  }, [builderService, isSorted, language, outputPath, sourcesPath]);

  const onCancel = useCallback(() => builderService.cancel(), [builderService]);

  const onLanguageChanged = useCallback(
    (event: SelectChangeEvent<string>) => {
      setLanguage(event.target.value);
    },
    [setLanguage]
  );

  // Anything the build depends on invalidates whatever the previous run reported.
  useEffect(() => {
    builderService.reset();
  }, [sourcesPath, outputPath, language, isSorted, builderService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid || !destination.isValid}
      title={"Build translations"}
      description={"Compiles JSON sources into one X-Ray string table per language, in each language's code page."}
      error={builderService.error ?? undefined}
      submitLabel={"Build"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        builderService.result ? (
          <TranslationsBuildResult result={builderService.result} outputPath={outputPath} />
        ) : null
      }
      onSubmit={onBuild}
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
        controlId={"translations-builder-language"}
        isInline
      >
        <Select
          id={"translations-builder-language"}
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

      <PathFormRow
        isDisabled={isRunning}
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
          disabled={isRunning}
          onChange={(event) => setIsSorted(event.target.checked)}
        />
      </FormRow>
    </PickerForm>
  );
}
