import { Box } from "@mui/material";
import { GridColDef, GridRenderCellParams } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { describePatchHeadline } from "@/applications/archives-patcher/lib/describe-patch-headline";
import { type IPatchChangeRow, toPatchChangeRows } from "@/applications/archives-patcher/lib/patch-change-rows";
import { ArchivePatchClass, ArchivePatchResult } from "@/core/bindings/types/xrf-pack";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

const CHANGE_LABELS: Record<ArchivePatchClass, string> = {
  added: "Added",
  modified: "Modified",
};

/** Colour per class, so the table is read by scanning rather than by reading every label. */
const CHANGE_COLORS: Record<ArchivePatchClass, string> = {
  added: "success.main",
  modified: "warning.main",
};

/** Read from is off by default: two loose trees repeat one of two roots on every row. */
const HIDDEN_COLUMNS: Array<string> = ["origin"];

interface IArchivesPatchResultProps {
  result: ArchivePatchResult;
  /** Where the run was told to publish, for revealing what it wrote. */
  outputPath: Nullable<string>;
}

export function ArchivesPatchResult({ result, outputPath }: IArchivesPatchResultProps): ReactElement {
  const rows: Array<IPatchChangeRow> = useMemo(() => toPatchChangeRows(result), [result]);

  const columns: Array<GridColDef<IPatchChangeRow>> = useMemo(
    () => [
      {
        field: "class",
        headerName: "Change",
        width: 110,
        renderCell: (params: GridRenderCellParams<IPatchChangeRow>) => (
          <Box sx={{ display: "flex", alignItems: "center", height: "100%", color: CHANGE_COLORS[params.row.class] }}>
            {CHANGE_LABELS[params.row.class]}
          </Box>
        ),
      },
      {
        field: "size",
        headerName: "Size",
        width: 100,
        // Numeric so the column sorts on the byte count. Formatting it into the row would sort "9 KB" above "1 MB".
        type: "number",
        valueFormatter: (value: number) => formatBytes(value),
        cellClassName: "monospace",
      },
      { field: "name", headerName: "Entry", flex: 1, minWidth: 320, cellClassName: "monospace" },
      { field: "origin", headerName: "Read from", flex: 1, minWidth: 240, cellClassName: "monospace" },
    ],
    []
  );

  const isPublished: boolean = result.publication.kind === "published";

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "added", value: result.added.length },
      { label: "modified", value: result.modified.length },
      { label: "unchanged", value: result.unchanged },
      // Known before anything is written, which is what makes it worth showing on a preview.
      { label: isPublished ? "carried" : "to carry", value: formatBytes(result.sizeCarried) },
      { label: "payloads read", value: result.payloadsRead },
      { label: "compare", value: formatDuration(result.compareDuration) },
      { label: "elapsed", value: formatDuration(result.duration) },
    ],
    [isPublished, result]
  );

  return (
    <CommandResult
      headline={describePatchHeadline(result)}
      tone={"success"}
      stats={stats}
      actions={
        isPublished ? (
          <RevealPathButton application={EApplicationId.ARCHIVES_PATCHER} path={outputPath} label={"Show patch"} />
        ) : null
      }
    >
      <CommandResultFindings<IPatchChangeRow>
        rows={rows}
        columns={columns}
        getRowId={(row) => `${row.class}:${row.name}`}
        getSearchText={(row) => row.name}
        hiddenColumns={HIDDEN_COLUMNS}
        emptyLabel={"Nothing differs between the two sides."}
        searchPlaceholder={"Filter by entry"}
      />
    </CommandResult>
  );
}
