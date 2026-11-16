import { useInjection } from "@wirestate/react";
import { ETranslationLanguage } from "@xrf/ipc/types/xrf-translation";
import { Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { SwitchFormRow } from "@xrf/ui/form/switch-row";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { TranslationLanguageField } from "@/core/translations/components/TranslationLanguageField";
import { DEFAULT_TRANSLATION_LANGUAGE, TRANSLATION_LANGUAGES } from "@/core/translations/translations.config";
import { IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { TranslationsParserService } from "@/tools/translations-parser/services/translations-parser.service";

import { TranslationsParseResult } from "./components/TranslationsParseResult";

interface ITranslationsParseRequest {
  source: string;
  /** The language the raw XML is declared to be in. */
  language: ETranslationLanguage;
  output: string;
  isOverwrite: boolean;
}

export function TranslationsParserTool(): ReactElement {
  const parserService: TranslationsParserService = useInjection(TranslationsParserService);

  // The run rather than this view's own flag: an import survives the window being reloaded.
  const isRunning: boolean = parserService.operation.isRunning;

  const [isOverwrite, setIsOverwrite] = useState<boolean>(false);

  // The language is remembered, because it says which translations are being worked on. The overwrite
  // switch deliberately is not: it decides whether a run may replace somebody else's text.
  const [language, setLanguage] = useRememberedValue({
    tool: EToolId.TRANSLATIONS_PARSER,
    id: "language",
    fallback: DEFAULT_TRANSLATION_LANGUAGE,
    allowed: TRANSLATION_LANGUAGES,
  });

  const source: IPathField = usePathField({
    tool: EToolId.TRANSLATIONS_PARSER,
    id: "source",
    title: "Select translations source",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.TRANSLATIONS_PARSER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const outputPath: Nullable<string> = destination.value;

  const request: Nullable<ITranslationsParseRequest> = useMemo(
    () => (source.value && outputPath ? { source: source.value, language, output: outputPath, isOverwrite } : null),
    [isOverwrite, language, outputPath, source.value]
  );

  const onRun = useCallback(
    async ({ submitted, isDryRun }: { submitted: ITranslationsParseRequest; isDryRun: boolean }): Promise<void> => {
      await parserService.parse(
        submitted.source,
        submitted.language,
        submitted.output,
        submitted.isOverwrite,
        isDryRun
      );
    },
    [parserService]
  );

  const onPreviewClicked = useCallback(() => {
    if (request) {
      void onRun({ submitted: request, isDryRun: true });
    }
  }, [onRun, request]);

  const onImport = useCallback(
    (submitted: ITranslationsParseRequest) => void onRun({ submitted, isDryRun: false }),
    [onRun]
  );

  return (
    <JobPickerForm
      operation={parserService.operation}
      request={request}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Parse translations"}
      description={
        "Reads one language's raw XML string tables and merges them into JSON sources, filling gaps with placeholders."
      }
      submitLabel={"Import"}
      secondaryActions={
        <Button
          variant={"outlined"}
          disabled={isRunning || !request || !source.isValid || !destination.isValid}
          onClick={onPreviewClicked}
        >
          Preview
        </Button>
      }
      renderResult={(result) => <TranslationsParseResult result={result} outputPath={outputPath} />}
      onSubmit={onImport}
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
