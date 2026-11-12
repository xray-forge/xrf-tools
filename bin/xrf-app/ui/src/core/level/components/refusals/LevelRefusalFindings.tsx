import { LevelSceneRefusal } from "@xrf/ipc/types/xrf-rawdata";
import { CommandResultFindings } from "@xrf/ui/command-result";
import { GridColDef } from "@xrf/ui/data-grid";
import { pathColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

interface ILevelRefusalRow extends LevelSceneRefusal {
  /** The row's place in the run's list, since two refusals may name the same object of the same file. */
  index: number;
}

interface ILevelRefusalFindingsProps {
  refused: Array<LevelSceneRefusal>;
  /** What the list is of when it is empty, such as "Nothing was left out of the scene.". */
  emptyLabel: string;
}

/**
 * What a level decompile or build left out, a row per object with the file it came from and why: the table both level
 * tools end on, searchable by any of the three.
 */
export function LevelRefusalFindings({ refused, emptyLabel }: ILevelRefusalFindingsProps): ReactElement {
  const rows: Array<ILevelRefusalRow> = useMemo(
    () => refused.map((refusal: LevelSceneRefusal, index: number) => ({ ...refusal, index })),
    [refused]
  );

  const columns: Array<GridColDef<ILevelRefusalRow>> = useMemo(
    () => [
      pathColumn("file", "File", 160),
      pathColumn("object", "Object", 260),
      { field: "reason", headerName: "Why", flex: 1 },
    ],
    []
  );

  return (
    <CommandResultFindings<ILevelRefusalRow>
      rows={rows}
      columns={columns}
      getRowId={(row) => row.index}
      getSearchText={(row) => `${row.file} ${row.object} ${row.reason}`}
      emptyLabel={emptyLabel}
      searchPlaceholder={"Filter by file, object or reason"}
    />
  );
}
