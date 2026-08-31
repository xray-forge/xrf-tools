import { Button, MenuItem, Select, SelectChangeEvent, Switch } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsParseResult } from "@/applications/translations-parser/components/TranslationsParseResult";
import { TranslationsParserService } from "@/applications/translations-parser/services/parser";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { DEFAULT_TRANSLATION_LANGUAGE, TRANSLATION_LANGUAGES } from "@/core/translations";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";
import { Nullable } from "@/lib/types/general";

export function TranslationsParserApplication(): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);
  const parserService: TranslationsParserService = useInjection(TranslationsParserService);

  // The run rather than this view's own flag: an import survives the window being reloaded.
  const job: Nullable<IJobState> = parserService.job;
  const isRunning: boolean = Boolean(job);

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
    seed: () => resolveExistingPathRole(EPathRole.BUILT_TRANSLATIONS, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_PARSER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
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

  const onCancel = useCallback(() => parserService.cancel(), [parserService]);

  const onPreviewClicked = useCallback(() => void onRun(true), [onRun]);

  const onImportClicked = useCallback(() => void onRun(false), [onRun]);

  const onLanguageChanged = useCallback(
    (event: SelectChangeEvent<string>) => {
      setLanguage(event.target.value);
    },
    [setLanguage]
  );

  // Changing anything the run depends on invalidates whatever the previous one reported.
  useEffect(() => {
    parserService.reset();
  }, [sourcePath, outputPath, language, isOverwrite, parserService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Parse translations"}
      description={
        "Reads one language's raw XML string tables and merges them into JSON sources, filling gaps with placeholders."
      }
      error={parserService.error ?? undefined}
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
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        parserService.result ? <TranslationsParseResult result={parserService.result} outputPath={outputPath} /> : null
      }
      onSubmit={onImportClicked}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Source"}
        description={"Mod folder, gamedata tree, or game installation holding the XML string tables"}
        field={source}
      />

      <FormRow
        label={"Language"}
        description={"Raw XML carries no language, so it is declared rather than guessed"}
        controlId={"translations-parser-language"}
        isInline
      >
        <Select
          id={"translations-parser-language"}
          size={"small"}
          value={language}
          disabled={isRunning}
          onChange={onLanguageChanged}
        >
          {TRANSLATION_LANGUAGES.map((it) => (
            <MenuItem key={it} value={it}>
              {it}
            </MenuItem>
          ))}
        </Select>
      </FormRow>

      <PathFormRow
        isDisabled={isRunning}
        label={"Output"}
        description={"Directory the JSON sources are written to, merging with any already there"}
        field={destination}
      />

      <FormRow
        label={"Replace existing text"}
        description={"Text already in the output is kept unless this is on"}
        controlId={"translations-parser-overwrite"}
        isRequired={false}
        isInline
      >
        <Switch
          id={"translations-parser-overwrite"}
          size={"small"}
          checked={isOverwrite}
          disabled={isRunning}
          onChange={(event) => setIsOverwrite(event.target.checked)}
        />
      </FormRow>
    </PickerForm>
  );
}
