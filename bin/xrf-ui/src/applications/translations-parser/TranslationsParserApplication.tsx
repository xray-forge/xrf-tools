import { Button, MenuItem, Select, SelectChangeEvent, Switch } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsParseResult } from "@/applications/translations-parser/components/TranslationsParseResult";
import { translationsCommands } from "@/core/bindings/commands/translations";
import { TranslationParseSummary } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { DEFAULT_TRANSLATION_LANGUAGE, TRANSLATION_LANGUAGES } from "@/core/translations";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function TranslationsParserApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const pathsService: PathsService = useInjection(PathsService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<TranslationParseSummary>>(null);
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
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.BUILT_TRANSLATIONS, pathsService.paths),
  });

  const destination: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_PARSER,
    id: "destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isLoading,
    seed: () => resolvePathRole(EPathRole.TRANSLATIONS, pathsService.paths),
  });

  const sourcePath: Nullable<string> = source.value;
  const outputPath: Nullable<string> = destination.value;

  const onRun = useCallback(
    async (isDryRun: boolean): Promise<void> => {
      if (!sourcePath || !outputPath) {
        return;
      }

      try {
        setIsLoading(true);
        setResult(null);
        setError(null);

        log.info("Parsing translations:", sourcePath, language, isDryRun ? "(preview)" : "");

        const roots: XrayRoots = { asset: null, roots: [{ path: sourcePath, mode: "containingInstallation" }] };

        const summary: TranslationParseSummary = await translationsCommands.parseProject(
          roots,
          language,
          null,
          outputPath,
          null,
          isOverwrite,
          isDryRun
        );

        setResult(summary);

        notify({
          details: `${sourcePath}\n${outputPath}`,
          severity: ENotificationSeverity.SUCCESS,
          source: EApplicationId.TRANSLATIONS_PARSER,
          title: isDryRun ? "Previewed translations import" : "Imported translations",
        });
      } catch (error: unknown) {
        log.error("Parse error:", error);
        setError(String(error));

        notify({
          details: `${sourcePath}\n${String(error)}`,
          severity: ENotificationSeverity.ERROR,
          source: EApplicationId.TRANSLATIONS_PARSER,
          title: "Could not import translations",
        });
      } finally {
        setIsLoading(false);
      }
    },
    [sourcePath, outputPath, language, isOverwrite, log, notify]
  );

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
    setError(null);
    setResult(null);
  }, [sourcePath, outputPath, language, isOverwrite]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!source.isValid || !destination.isValid}
      title={"Parse translations"}
      description={
        "Reads one language's raw XML string tables and merges them into JSON sources, filling gaps with placeholders."
      }
      error={error ?? undefined}
      submitLabel={"Import"}
      secondaryActions={
        <Button
          variant={"outlined"}
          disabled={isLoading || !source.isValid || !destination.isValid}
          onClick={onPreviewClicked}
        >
          Preview
        </Button>
      }
      result={result ? <TranslationsParseResult result={result} outputPath={outputPath} /> : null}
      onSubmit={onImportClicked}
    >
      <PathFormRow
        isDisabled={isLoading}
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
          disabled={isLoading}
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
        isDisabled={isLoading}
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
          disabled={isLoading}
          onChange={(event) => setIsOverwrite(event.target.checked)}
        />
      </FormRow>
    </PickerForm>
  );
}
