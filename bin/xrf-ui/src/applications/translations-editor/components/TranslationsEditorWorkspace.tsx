import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { TranslationsFilesMenu } from "@/applications/translations-editor/components/editor/TranslationsFilesMenu";
import { TranslationsLanguageBar } from "@/applications/translations-editor/components/editor/TranslationsLanguageBar";
import {
  ITranslationRow,
  TranslationsTable,
} from "@/applications/translations-editor/components/editor/TranslationsTable";
import {
  ITranslationValidation,
  useTranslationValidation,
} from "@/applications/translations-editor/lib/use-translation-validation";
import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { TranslationFile, TranslationProjectDescriptor } from "@/core/bindings/types/xrf-translation";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { Nullable } from "@/lib/types/general";

export function TranslationsEditorWorkspace(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  const project: Nullable<TranslationProjectDescriptor> = translationsService.project.value;
  const languages: Array<string> = useMemo(() => project?.languages ?? [], [project]);

  const [selectedFile, setSelectedFile] = useState<Nullable<string>>(null);
  const [selectedId, setSelectedId] = useState<Nullable<string>>(null);
  const [reference, setReference] = useState<string>("");
  const [target, setTarget] = useState<string>("");

  const { getErrorOf, validate }: ITranslationValidation = useTranslationValidation({
    file: selectedFile,
    language: target,
    validateText: (language: string, text: string) => translationsService.validateText(language, text),
  });

  const file: Nullable<TranslationFile> = selectedFile ? (project?.files[selectedFile] ?? null) : null;

  // Not memoised: the values come from observables, so a cache on props would keep rows from before the last edit.
  const rows: Array<ITranslationRow> =
    selectedFile && file
      ? Object.keys(file.entries).map((id: string): ITranslationRow => {
          const pending: Record<string, Nullable<string>> | undefined =
            translationsService.edits[selectedFile]?.[target];

          return {
            id,
            reference: translationsService.resolveValue(selectedFile, reference, id),
            target: translationsService.resolveValue(selectedFile, target, id),
            isEdited: Boolean(pending && id in pending),
            error: getErrorOf(id),
          };
        })
      : [];

  const onCommit = useCallback(
    (id: string, value: string) => {
      if (!selectedFile) {
        return;
      }

      translationsService.setEdit(selectedFile, target, id, value);

      // Asked at commit rather than at save, so a character the code page cannot hold is reported
      // where it was typed instead of at the end of a batch.
      validate(id, value);
    },
    [selectedFile, target, translationsService, validate]
  );

  useEffect(() => {
    const files: Array<string> = Object.keys(project?.files ?? {});

    setSelectedFile((current: Nullable<string>) => (current && files.includes(current) ? current : (files[0] ?? null)));
    setReference((current: string) => (languages.includes(current) ? current : (languages[0] ?? "")));
    setTarget((current: string) => (languages.includes(current) ? current : (languages[1] ?? languages[0] ?? "")));
  }, [languages, project]);

  if (!project) {
    return <EmptyState title={"Nothing open"} description={"Open a translations directory to edit it."} />;
  }

  return (
    <Box sx={{ display: "flex", width: "100%", height: "100%", minHeight: 0 }}>
      <Box sx={{ width: 280, flexShrink: 0, borderRight: 1, borderColor: "divider" }}>
        <TranslationsFilesMenu
          files={project.files}
          dirtyFiles={translationsService.dirtyFiles}
          selected={selectedFile}
          onSelect={setSelectedFile}
        />
      </Box>

      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          flexGrow: 1,
          minWidth: 0,
          minHeight: 0,
          gap: 1,
          padding: 1.5,
        }}
      >
        <TranslationsLanguageBar
          languages={languages}
          encodings={project.encodings}
          reference={reference}
          target={target}
          onReferenceChange={setReference}
          onTargetChange={setTarget}
        />

        {file ? (
          <TranslationsTable
            rows={rows}
            targetLanguage={target}
            isDisabled={translationsService.savingFile !== null}
            selectedId={selectedId}
            onCommit={onCommit}
            onSelect={setSelectedId}
          />
        ) : (
          <EmptyState title={"Select a file"} description={"Pick a translation file to see its entries."} />
        )}
      </Box>
    </Box>
  );
}
