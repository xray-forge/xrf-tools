import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { Patrol } from "@/core/ipc/types/xrf-db";
import { identifierColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPatrolRow {
  name: string;
  pointsCount: number;
  linksCount: number;
}

interface ISpawnEditorPatrolsTableProps extends BaseComponentProps {
  patrols: Array<Patrol>;
}

export function SpawnEditorPatrolsTable({
  "data-testid": dataTestId = "spawn-editor-patrols-table",
  id,
  className,
  patrols,
}: ISpawnEditorPatrolsTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("name", "Patrol", 320),
      textColumn("pointsCount", "Points", 110),
      textColumn("linksCount", "Links", 110),
    ],
    []
  );

  const rows: Array<IPatrolRow> = useMemo(
    () =>
      patrols.map((it: Patrol) => ({
        linksCount: it.links.length,
        name: it.name,
        pointsCount: it.points.length,
      })),
    [patrols]
  );

  return (
    <SpawnTable<IPatrolRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"patrol"}
      emptyLabel={"This file defines no patrols."}
      source={"Patrol"}
      getRowId={(row: IPatrolRow): GridRowId => row.name}
      getSearchText={(row: IPatrolRow): string => row.name}
    />
  );
}
