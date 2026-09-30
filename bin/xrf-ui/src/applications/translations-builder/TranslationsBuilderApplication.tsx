import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsBuilderService } from "@/applications/translations-builder/services/builder";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, SwitchFormRow, usePathField, useRememberedValue } from "@/core/ui/form";

import { TranslationsBuildResult } from "./components/TranslationsBuildResult";

export function TranslationsBuilderApplication(): ReactElement {
  const builderService: TranslationsBuilderService = useInjection(TranslationsBuilderService);

  // The run rather than this view's own flag: a build survives the window being reloaded.
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

  // Anything the build depends on invalidates whatever the previous run reported.
  useEffect(() => {
    builderService.operation.reset();
  }, [sourcesPath, outputPath, language, isSorted, builderService]);

  return (
    <JobPickerForm
      operation={builderService.operation}
      isSubmitDisabled={!sources.isValid || !destination.isValid}
      title={"Build translations"}
      description={"Compiles JSON sources into one X-Ray string table per language, in each language's code page."}
      submitLabel={"Build"}
      renderResult={(result) => <TranslationsBuildResult result={result} outputPath={outputPath} />}
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

      <SwitchFormRow
        id={"translations-builder-sort"}
        label={"Sort ids"}
        description={"Off preserves the order each source declares them in"}
        isChecked={isSorted}
        isDisabled={isRunning}
        isRequired={false}
        onChange={setIsSorted}
      />
    </JobPickerForm>
  );
}
