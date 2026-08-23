import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { GraphVertex } from "@/core/bindings/types/xrf-db";
import { textColumn, tupleColumn, vectorColumn } from "@/core/ui/table";

/** Offsets locate a vertex inside the file rather than in the roots; available, off by default. */
const HIDDEN_COLUMNS: Array<string> = ["edgesOffset", "levelPointsOffset"];

interface IGraphVertexRow extends GraphVertex {
  index: number;
}

interface ISpawnEditorGraphVerticesTableProps {
  vertices: Array<GraphVertex>;
}

export function SpawnEditorGraphVerticesTable({ vertices }: ISpawnEditorGraphVerticesTableProps): ReactElement {
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
