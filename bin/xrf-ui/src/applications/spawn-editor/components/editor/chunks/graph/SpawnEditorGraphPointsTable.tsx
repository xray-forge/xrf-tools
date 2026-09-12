import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { GraphLevelPoint } from "@/core/bindings/types/xrf-db";
import { decimalColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IGraphPointRow extends GraphLevelPoint {
  index: number;
}

interface ISpawnEditorGraphPointsTableProps extends BaseComponentProps {
  points: Array<GraphLevelPoint>;
}

export function SpawnEditorGraphPointsTable({
  "data-testid": dataTestId = "spawn-editor-graph-points-table",
  id,
  className,
  points,
}: ISpawnEditorGraphPointsTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("index", "#", 90),
      textColumn("levelVertexId", "Level vertex", 140),
      decimalColumn("distance", "Distance", 130),
    ],
    []
  );

  const rows: Array<IGraphPointRow> = useMemo(
    () => points.map((it: GraphLevelPoint, index: number) => ({ ...it, index })),
    [points]
  );

  return (
    <SpawnTable<IGraphPointRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"point"}
      emptyLabel={"This graph has no points."}
      source={"Graph point"}
      getRowId={(row: IGraphPointRow): GridRowId => row.index}
    />
  );
}
