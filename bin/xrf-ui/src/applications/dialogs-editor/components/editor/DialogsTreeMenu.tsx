import { Box, Typography } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { IDialogSelection } from "@/applications/dialogs-editor/services/dialogs";
import { DialogFileDescriptor, DialogSummaryDescriptor } from "@/core/bindings/types/xrf-dialog";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
import { Nullable } from "@/lib/types/general";

/** One dialog, flattened with the file holding it so a filter ranks across the whole project. */
interface IDialogEntry {
  logicalPath: string;
  fileName: string;
  id: string;
  phrases: number;
}

export interface IDialogsTreeMenuProps {
  files: Record<string, DialogFileDescriptor>;
  selection: Nullable<IDialogSelection>;
  onSelect: (logicalPath: string, id: string) => void;
}

/**
 * Every dialog in the project, filterable by name.
 *
 * Flat rather than a collapsible file tree. Five hundred dialogs across four files is a list you
 * search, not one you navigate: the filter is the way in, and the file name rides along on each row
 * so a match still says where it came from.
 */
export function DialogsTreeMenu({ files, selection, onSelect }: IDialogsTreeMenuProps): ReactElement {
  const entries: Array<IDialogEntry> = useMemo(
    () =>
      Object.entries(files).flatMap(([logicalPath, file]: [string, DialogFileDescriptor]) => {
        const fileName: string = logicalPath.split("\\").pop() ?? logicalPath;

        return file.dialogs.map(
          (dialog: DialogSummaryDescriptor): IDialogEntry => ({
            fileName,
            id: dialog.id,
            logicalPath,
            phrases: dialog.phrases,
          })
        );
      }),
    [files]
  );

  const search: IUseRankedSearch<IDialogEntry> = useRankedSearch({
    items: entries,
    toSearchText: (it: IDialogEntry) => it.id,
    onSelect: (it: IDialogEntry) => onSelect(it.logicalPath, it.id),
  });

  const visible: Array<IDialogEntry> = search.isSearching
    ? search.results.map((result: ISearchResult<IDialogEntry>) => result.item)
    : entries;

  const sections: Array<IEditorSideMenuItem> = visible.map((it: IDialogEntry) => ({
    label: it.id,
    description: `${it.fileName} · ${it.phrases} ${it.phrases === 1 ? "phrase" : "phrases"}`,
    isSelected: selection?.logicalPath === it.logicalPath && selection.id === it.id,
    onClick: () => onSelect(it.logicalPath, it.id),
  }));

  return (
    <EditorSideMenu
      header={
        <EditorSearchHeader
          title={"Dialogs"}
          count={entries.length}
          query={search.query}
          placeholder={"Filter dialogs"}
          ariaLabel={"Filter dialogs"}
          onClear={search.clear}
          onKeyDown={search.onInputKeyDown}
          onQueryChange={search.setQuery}
        />
      }
      sections={sections}
    >
      {visible.length ? null : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            No dialogs match {search.query.trim()}.
          </Typography>
        </Box>
      )}
    </EditorSideMenu>
  );
}
