import { default as CircleIcon } from "@mui/icons-material/Circle";
import { ReactElement, useCallback, useMemo } from "react";

import { TranslationFile } from "@/core/ipc/types/xrf-translation";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
import { EmptyListing } from "@/core/ui/layout";
import { Nullable } from "@/lib/types/general";

/** One file, with enough on it to rank a search and label a row. */
interface IFileEntry {
  name: string;
  entryCount: number;
  isDirty: boolean;
}

/** Describes a file consistently in the default list and search results. */
function toFileRow(entry: IFileEntry): IEditorSearchResultRow {
  return {
    id: entry.name,
    label: entry.name,
    description: `${entry.entryCount} entries`,
    // The 8px dot names no step of the type scale, so its size stays a raw value.
    icon: entry.isDirty ? (
      <CircleIcon aria-label={"Unsaved changes"} className={"text-badge text-warning"} />
    ) : undefined,
  };
}

export interface ITranslationsFilesMenuProps {
  files: Record<string, TranslationFile>;
  dirtyFiles: ReadonlyArray<string>;
  selected: Nullable<string>;
  onSelect: (file: string) => void;
}

export function TranslationsFilesMenu({
  files,
  dirtyFiles,
  selected,
  onSelect,
}: ITranslationsFilesMenuProps): ReactElement {
  const entries: Array<IFileEntry> = useMemo(
    () =>
      Object.entries(files).map(([name, file]: [string, TranslationFile]) => ({
        name,
        entryCount: Object.keys(file.entries).length,
        isDirty: dirtyFiles.includes(name),
      })),
    [dirtyFiles, files]
  );

  const toFileSearchText = useCallback((entry: IFileEntry): string => entry.name, []);

  const sections: Array<IEditorSideMenuItem> = entries.map((entry: IFileEntry) => ({
    ...toFileRow(entry),
    isSelected: entry.name === selected,
    onClick: () => onSelect(entry.name),
  }));

  return (
    <EditorSearchMenu
      title={"Files"}
      searchLabel={"Filter translation files"}
      placeholder={"Filter files"}
      resultsLabel={"Translation file search results"}
      items={entries}
      toSearchText={toFileSearchText}
      toRow={toFileRow}
      onSelect={(entry) => onSelect(entry.name)}
      sections={sections}
    >
      {entries.length ? null : <EmptyListing label={"No translation files found."} />}
    </EditorSearchMenu>
  );
}
