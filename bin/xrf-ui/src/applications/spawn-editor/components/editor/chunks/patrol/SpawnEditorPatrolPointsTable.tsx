import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { Patrol, PatrolPoint } from "@/core/ipc/types/xrf-db";
import { flagsColumn, identifierColumn, textColumn, vectorColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPatrolPointRow extends PatrolPoint {
  id: string;
  patrol: string;
}

interface ISpawnEditorPatrolPointsTableProps extends BaseComponentProps {
  patrols: Array<Patrol>;
}

export function SpawnEditorPatrolPointsTable({
  "data-testid": dataTestId = "spawn-editor-patrol-points-table",
  id,
  className,
  patrols,
}: ISpawnEditorPatrolPointsTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("patrol", "Patrol", 300),
      identifierColumn("name", "Point", 180),
      flagsColumn("flags", "Flags"),
      textColumn("levelVertexId", "Level vertex", 130),
      textColumn("gameVertexId", "Game vertex", 130),
      vectorColumn("position", "Position"),
    ],
    []
  );

  const rows: Array<IPatrolPointRow> = useMemo(
    () =>
      patrols.flatMap((patrol: Patrol) =>
        patrol.points.map((point: PatrolPoint) => ({
          ...point,
          id: `${patrol.name}/${point.name}`,
          patrol: patrol.name,
        }))
      ),
    [patrols]
  );

  return (
    <SpawnTable<IPatrolPointRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"point"}
      emptyLabel={"These patrols have no points."}
      source={"Patrol point"}
      getRowId={(row: IPatrolPointRow): GridRowId => row.id}
      getSearchText={(row: IPatrolPointRow): string => `${row.patrol} ${row.name}`}
    />
  );
}
