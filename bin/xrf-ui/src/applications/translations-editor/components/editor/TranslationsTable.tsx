import { Tooltip, Typography } from "@mui/material";
import { DataGrid, GridColDef, GridRenderCellParams, GridRowParams } from "@mui/x-data-grid";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo, useState } from "react";

import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";

/** One id, as the pair of languages currently in view sees it. */
export interface ITranslationRow {
  id: string;
  reference: Nullable<string>;
  target: Nullable<string>;
  isEdited: boolean;
  error: Nullable<string>;
}

export interface ITranslationsTableProps {
  rows: Array<ITranslationRow>;
  targetLanguage: string;
  isDisabled?: boolean;
  onCommit: (id: string, value: string) => void;
  onSelect: (id: string) => void;
  selectedId: Nullable<string>;
}

function renderValue(value: Nullable<string>): ReactElement {
  // Absent is not empty: the engine falls back to the id, so the gap is a real state worth seeing.
  if (value === null) {
    return (
      <Typography className={"text-text-disabled italic"} variant={"body2"}>
        not translated
      </Typography>
    );
  }

  return (
    <Typography className={"truncate"} variant={"body2"}>
      {value}
    </Typography>
  );
}

const EDITED_ROW_CLASS: string = "xrf-translation-row-edited";

// todo: Split the file.
export function TranslationsTable({
  rows,
  targetLanguage,
  isDisabled,
  onCommit,
  onSelect,
  selectedId,
}: ITranslationsTableProps): ReactElement {
  const [search, setSearch] = useState<string>("");

  const filtered: Array<ITranslationRow> = useMemo(() => {
    const query: string = search.trim().toLowerCase();

    if (!query) {
      return rows;
    }

    return rows.filter(
      (row: ITranslationRow) =>
        row.id.toLowerCase().includes(query) ||
        (row.reference ?? "").toLowerCase().includes(query) ||
        (row.target ?? "").toLowerCase().includes(query)
    );
  }, [rows, search]);

  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "id", headerName: "Id", flex: 1, minWidth: 220, cellClassName: "monospace" },
      {
        field: "reference",
        headerName: "Reference",
        flex: 1,
        minWidth: 260,
        sortable: false,
        renderCell: (params: GridRenderCellParams<ITranslationRow>) => renderValue(params.row.reference),
      },
      {
        field: "target",
        headerName: `Target · ${targetLanguage}`,
        flex: 1,
        minWidth: 260,
        sortable: false,
        editable: !isDisabled,
        valueGetter: (_, row: ITranslationRow) => row.target ?? "",
        renderCell: (params: GridRenderCellParams<ITranslationRow>) =>
          params.row.error ? (
            <Tooltip describeChild title={params.row.error}>
              <div className={"flex w-full items-center text-error"}>{renderValue(params.row.target)}</div>
            </Tooltip>
          ) : (
            renderValue(params.row.target)
          ),
      },
    ],
    [isDisabled, targetLanguage]
  );

  return (
    <div className={"flex min-h-0 grow flex-col gap-2"}>
      <div className={"max-w-80"}>
        <EditorFilterInput
          ariaLabel={"Filter translations"}
          query={search}
          placeholder={"Filter by id or text"}
          onQueryChange={setSearch}
        />
      </div>

      <DataGrid
        className={"min-h-0 grow"}
        rows={filtered}
        columns={columns}
        getRowId={(row: ITranslationRow) => row.id}
        density={"compact"}
        disableRowSelectionOnClick={false}
        rowSelectionModel={selectedId ? { type: "include", ids: new Set([selectedId]) } : undefined}
        initialState={{ pagination: { paginationModel: { pageSize: 100 } } }}
        pageSizeOptions={[100, 250, 500]}
        sx={{ [`& .${EDITED_ROW_CLASS}`]: { backgroundColor: "action.hover" } }}
        getRowClassName={(params: GridRowParams<ITranslationRow>) => (params.row.isEdited ? EDITED_ROW_CLASS : "")}
        processRowUpdate={(updated: ITranslationRow, original: ITranslationRow) => {
          if (updated.target !== original.target) {
            onCommit(updated.id, updated.target ?? "");
          }

          return updated;
        }}
        onRowClick={(params: GridRowParams<ITranslationRow>) => onSelect(params.row.id)}
      />
    </div>
  );
}
