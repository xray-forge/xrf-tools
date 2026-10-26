import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsParserService } from "@/applications/translations-parser/services/parser";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { DEFAULT_TRANSLATION_LANGUAGE, TRANSLATION_LANGUAGES } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, SwitchFormRow, usePathField, useRememberedValue } from "@/core/ui/form";

import { TranslationsParseResult } from "./components/TranslationsParseResult";

export function TranslationsParserApplication(): ReactElement {
  const parserService: TranslationsParserService = useInjection(TranslationsParserService);

  // The run rather than this view's own flag: an import survives the window being reloaded.
  const isRunning: boolean = parserService.operation.isRunning;

  const [isOverwrite, setIsOverwrite] = useState<boolean>(false);

  // The language is remembered, because it says which translations are being worked on. The overwrite
  // switch deliberately is not: it decides whether a run may replace somebody else's text.
  const [language, setLanguage] = useRememberedValue({
    application: EApplicationId.TRANSLATIONS_PARSER,
    id: "language",
    fallback: DEFAULT_TRANSLATION_LANGUAGE,
    allowed: TRANSLATION_LANGUAGES,
  });

  const source: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_PARSER,
    id: "source",
    title: "Select translations source",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_PARSER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const sourcePath: Nullable<string> = source.value;
  const outputPath: Nullable<string> = destination.value;

  const onRun = useCallback(
    async (isDryRun: boolean): Promise<void> => {
      if (!sourcePath || !outputPath) {
        return;
      }

      await parserService.parse(sourcePath, language, outputPath, isOverwrite, isDryRun);
    },
    [isOverwrite, language, outputPath, parserService, sourcePath]
  );

  const onPreviewClicked = useCallback(() => void onRun(true), [onRun]);

  const onImportClicked = useCallback(() => void onRun(false), [onRun]);

  // Changing anything the run depends on invalidates whatever the previous one reported.
  useEffect(() => {
    parserService.operation.reset();
  }, [sourcePath, outputPath, language, isOverwrite, parserService]);

  return (
    <JobPickerForm
      operation={parserService.operation}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Parse translations"}
      description={
        "Reads one language's raw XML string tables and merges them into JSON sources, filling gaps with placeholders."
      }
      submitLabel={"Import"}
      secondaryActions={
        <Button
          variant={"outlined"}
          disabled={isRunning || !source.isValid || !destination.isValid}
          onClick={onPreviewClicked}
        >
          Preview
        </Button>
      }
      renderResult={(result) => <TranslationsParseResult result={result} outputPath={outputPath} />}
      onSubmit={onImportClicked}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Source"}
        description={"Mod folder, gamedata tree, or game installation holding the XML string tables"}
        field={source}
      />

      <TranslationLanguageField
        id={"translations-parser-language"}
        description={"Raw XML carries no language, so it is declared rather than guessed"}
        value={language}
        isDisabled={isRunning}
        onChange={setLanguage}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Output"}
        description={"Directory the JSON sources are written to, merging with any already there"}
        field={destination}
      />

      <SwitchFormRow
        id={"translations-parser-overwrite"}
        label={"Replace existing text"}
        description={"Text already in the output is kept unless this is on"}
        isChecked={isOverwrite}
        isDisabled={isRunning}
        isRequired={false}
        onChange={setIsOverwrite}
      />
    </JobPickerForm>
  );
}
