import { default as CircleIcon } from "@mui/icons-material/Circle";
import { Box, Typography } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { TranslationFile } from "@/core/bindings/types/xrf-translation";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
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
    icon: entry.isDirty ? (
      <CircleIcon aria-label={"Unsaved changes"} sx={{ fontSize: 8, color: "warning.main" }} />
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
      toSearchText={(entry) => entry.name}
      toRow={toFileRow}
      onSelect={(entry) => onSelect(entry.name)}
      sections={sections}
    >
      {entries.length ? null : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            No translation files found.
          </Typography>
        </Box>
      )}
    </EditorSearchMenu>
  );
}
