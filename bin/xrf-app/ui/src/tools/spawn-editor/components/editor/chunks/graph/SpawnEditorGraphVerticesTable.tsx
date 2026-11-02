import { GraphVertex } from "@xrf/ipc/types/xrf-spawn";
import { GridColDef, GridRowId } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { textColumn, tupleColumn, vectorColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/tools/spawn-editor/components/editor/table/SpawnTable";

/** Offsets locate a vertex inside the file rather than in the roots; available, off by default. */
const HIDDEN_COLUMNS: Array<string> = ["edgesOffset", "levelPointsOffset"];

interface IGraphVertexRow extends GraphVertex {
  index: number;
}

interface ISpawnEditorGraphVerticesTableProps extends BaseComponentProps {
  vertices: Array<GraphVertex>;
}

export function SpawnEditorGraphVerticesTable({
  "data-testid": dataTestId = "spawn-editor-graph-vertices-table",
  id,
  className,
  vertices,
}: ISpawnEditorGraphVerticesTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("index", "#", 90),
      textColumn("levelId", "Level", 90),
      textColumn("levelVertexId", "Level vertex", 130),
      vectorColumn("gamePoint", "Game point"),
      vectorColumn("levelPoint", "Level point"),
      textColumn("edgesCount", "Edges", 100),
      textColumn("levelPointsCount", "Points", 100),
      tupleColumn("vertexType", "Vertex type"),
      textColumn("edgesOffset", "Edges offset", 130),
      textColumn("levelPointsOffset", "Points offset", 130),
    ],
    []
  );

  const rows: Array<IGraphVertexRow> = useMemo(
    () => vertices.map((it: GraphVertex, index: number) => ({ ...it, index })),
    [vertices]
  );

  return (
    <SpawnTable<IGraphVertexRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"vertex"}
      emptyLabel={"This graph has no vertices."}
      hiddenColumns={HIDDEN_COLUMNS}
      source={"Graph vertex"}
      getRowId={(row: IGraphVertexRow): GridRowId => row.index}
    />
  );
}
