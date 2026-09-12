import { Switch } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsBuilderService } from "@/applications/translations-builder/services/builder";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { FormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

import { TranslationsBuildResult } from "./components/TranslationsBuildResult";

export function TranslationsBuilderApplication(): ReactElement {
  const builderService: TranslationsBuilderService = useInjection(TranslationsBuilderService);

  // The run rather than this view's own flag: a build survives the window being reloaded.
  const job: Nullable<IJobState> = builderService.operation.job;
  const isRunning: boolean = builderService.operation.isRunning;

  const [isSorted, setIsSorted] = useState<boolean>(true);
  const [language, setLanguage] = useRememberedValue({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: TRANSLATION_LANGUAGES_WITH_ALL,
  });

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_BUILDER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const sourcesPath: Nullable<string> = sources.value;
  const outputPath: Nullable<string> = destination.value;

  const onBuild = useCallback(async () => {
    if (!sourcesPath || !outputPath) {
      return;
    }

    await builderService.build(sourcesPath, language, outputPath, isSorted);
  }, [builderService, isSorted, language, outputPath, sourcesPath]);

  const onCancel = useCallback(() => builderService.operation.cancel(), [builderService]);

  // Anything the build depends on invalidates whatever the previous run reported.
  useEffect(() => {
    builderService.operation.reset();
  }, [sourcesPath, outputPath, language, isSorted, builderService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid || !destination.isValid}
      title={"Build translations"}
      description={"Compiles JSON sources into one X-Ray string table per language, in each language's code page."}
      error={builderService.operation.error ?? undefined}
      submitLabel={"Build"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        builderService.operation.result ? (
          <TranslationsBuildResult result={builderService.operation.result} outputPath={outputPath} />
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

      <TranslationLanguageField
        id={"translations-builder-language"}
        description={"One language, or every language the build compiles"}
        value={language}
        isAllAllowed
        isDisabled={isRunning}
        onChange={setLanguage}
      />

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
        {(props) => (
          <Switch
            slotProps={{ input: props }}
            size={"small"}
            checked={isSorted}
            disabled={isRunning}
            onChange={(event) => setIsSorted(event.target.checked)}
          />
        )}
      </FormRow>
    </PickerForm>
  );
}
