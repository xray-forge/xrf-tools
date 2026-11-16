import { useInjection } from "@wirestate/react";
import { ETranslationLanguage } from "@xrf/ipc/types/xrf-translation";
import { Nullable } from "@xrf/types";
import { SwitchFormRow } from "@xrf/ui/form/switch-row";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { ALL_TRANSLATION_LANGUAGES, TRANSLATION_LANGUAGES_WITH_ALL } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { TranslationsBuilderService } from "@/tools/translations-builder/services/translations-builder.service";

import { TranslationsBuildResult } from "./components/TranslationsBuildResult";

interface ITranslationsBuildRequest {
  sources: string;
  /** One language, or every language. */
  language: ETranslationLanguage;
  output: string;
  isSorted: boolean;
}

export function TranslationsBuilderTool(): ReactElement {
  const builderService: TranslationsBuilderService = useInjection(TranslationsBuilderService);

  // The run rather than this view's own flag: a build survives the window being reloaded.
  const isRunning: boolean = builderService.operation.isRunning;

  const [isSorted, setIsSorted] = useState<boolean>(true);
  const [language, setLanguage] = useRememberedValue({
    tool: EToolId.TRANSLATIONS_BUILDER,
    id: "language",
    fallback: ALL_TRANSLATION_LANGUAGES,
    allowed: TRANSLATION_LANGUAGES_WITH_ALL,
  });

  const sources: IPathField = usePathField({
    tool: EToolId.TRANSLATIONS_BUILDER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.TRANSLATIONS_BUILDER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const outputPath: Nullable<string> = destination.value;

  const request: Nullable<ITranslationsBuildRequest> = useMemo(
    () => (sources.value && outputPath ? { sources: sources.value, language, output: outputPath, isSorted } : null),
    [isSorted, language, outputPath, sources.value]
  );

  const onBuild = useCallback(
    async (submitted: ITranslationsBuildRequest) => {
      await builderService.build(submitted.sources, submitted.language, submitted.output, submitted.isSorted);
    },
    [builderService]
  );

  return (
    <JobPickerForm
      operation={builderService.operation}
      request={request}
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
