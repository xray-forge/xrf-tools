import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import {
  ITranslationValidation,
  useTranslationValidation,
} from "@/applications/translations-editor/lib/use-translation-validation";
import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { TranslationFile, TranslationProjectDescriptor } from "@/core/ipc/types/xrf-translation";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EmptyState } from "@/core/ui/layout/EmptyState";

import { TranslationsFilesMenu } from "./editor/TranslationsFilesMenu";
import { TranslationsLanguageBar } from "./editor/TranslationsLanguageBar";
import { ITranslationRow, TranslationsTable } from "./editor/TranslationsTable";

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
      ? Object.keys(file.entries).map((id: string): ITranslationRow => ({
          id,
          reference: translationsService.resolveValue(selectedFile, reference, id),
          target: translationsService.resolveValue(selectedFile, target, id),
          isEdited: translationsService.hasEdit(selectedFile, target, id),
          error: getErrorOf(id),
        }))
      : [];

  // Ends the selection only, leaving the project, its edits and the language pair alone. The row cursor goes with it:
  // it names an entry of the file being closed.
  const onDeselect = useCallback((): void => {
    setSelectedFile(null);
    setSelectedId(null);
  }, []);

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
    <div className={"flex h-full min-h-0 w-full"}>
      <div className={"w-70 shrink-0 border-r border-divider"}>
        <TranslationsFilesMenu
          files={project.files}
          dirtyFiles={translationsService.dirtyFiles}
          selected={selectedFile}
          onSelect={setSelectedFile}
        />
      </div>

      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {selectedFile ? (
          <EditorFileHeader
            data-testid={"translations-file-header"}
            name={selectedFile}
            caption={file ? `${Object.keys(file.entries).length} entries` : undefined}
            closeLabel={"Close file"}
            closeDescription={"Clear the selection and close this translation file"}
            onClose={onDeselect}
          />
        ) : null}

        <div className={"flex min-h-0 min-w-0 grow flex-col gap-2 p-3"}>
          <TranslationsLanguageBar
            languages={languages}
            encodings={project.encodings}
            reference={reference}
            target={target}
            onReferenceChange={setReference}
            onTargetChange={setTarget}
          />

          {selectedFile && file ? (
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
        </div>
      </div>
    </div>
  );
}
