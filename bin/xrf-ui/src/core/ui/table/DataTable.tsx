import { Box, TextField, Typography } from "@mui/material";
import { DataGrid, GridColDef, GridRowId, GridRowParams } from "@mui/x-data-grid";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IDataTableProps<T> extends BaseComponentProps {
  rows: Array<T>;
  columns: Array<GridColDef>;
  getRowId: (row: T) => GridRowId;
  /** Everything about a row a filter should match, flattened into one string. Omit for no filter. */
  getSearchText?: (row: T) => string;
  emptyLabel: string;
  /** Noun the row count is phrased with, singularised by the caller. */
  countNoun?: string;
  searchPlaceholder?: string;
  /** Column visibility the caller wants off by default; the columns panel still offers them. */
  hiddenColumns?: Array<string>;
  selectedRowId?: Nullable<GridRowId>;
  onRowSelect?: (row: T) => void;
}

export function DataTable<T>({
  "data-testid": dataTestId,
  id,
  className,
  rows,
  columns,
  getRowId,
  getSearchText,
  emptyLabel,
  countNoun = "row",
  searchPlaceholder = "Filter",
  hiddenColumns,
  selectedRowId,
  onRowSelect,
}: IDataTableProps<T>): ReactElement {
  const [search, setSearch] = useState<string>("");

  const filtered: Array<T> = useMemo(() => {
    const query: string = search.trim().toLowerCase();

    if (!query || !getSearchText) {
      return rows;
    }

    return rows.filter((row: T) => getSearchText(row).toLowerCase().includes(query));
    // `getSearchText` is declared inline by most callers, so depending on it would refilter every
    // render. Rows and the query are what actually change the result.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rows, search]);

  const initialState = useMemo(
    () => ({
      columns: {
        columnVisibilityModel: Object.fromEntries((hiddenColumns ?? []).map((field: string) => [field, false])),
      },
      pagination: { paginationModel: { pageSize: 50 } },
    }),
    // Read once by the grid, so a changing identity here would be ignored anyway.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );

  const onRowClick = useCallback((params: GridRowParams) => onRowSelect?.(params.row as T), [onRowSelect]);

  if (!rows.length) {
    return (
      <Typography
        data-testid={dataTestId}
        id={id}
        className={className}
        variant={"body2"}
        sx={{ color: "text.secondary" }}
      >
        {emptyLabel}
      </Typography>
    );
  }

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", gap: 1, width: "100%", flexGrow: 1, minHeight: 0 }}
    >
      {getSearchText ? (
        <TextField
          size={"small"}
          placeholder={searchPlaceholder}
          value={search}
          sx={{ maxWidth: 320 }}
          onChange={(event) => setSearch(event.target.value)}
        />
      ) : null}

      <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
        {filtered.length === rows.length
          ? `${rows.length} ${countNoun}(s)`
          : `${filtered.length} of ${rows.length} ${countNoun}(s)`}
      </Typography>

      <Box sx={{ flexGrow: 1, minHeight: 200, width: "100%" }}>
        <DataGrid
          columns={columns}
          getRowId={getRowId}
          initialState={initialState}
          pageSizeOptions={[25, 50, 100]}
          rowSelectionModel={
            selectedRowId === null || selectedRowId === undefined
              ? undefined
              : { type: "include", ids: new Set([selectedRowId]) }
          }
          rows={filtered}
          onRowClick={onRowSelect ? onRowClick : undefined}
        />
      </Box>
    </Box>
  );
}
