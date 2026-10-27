import { GraphHeader } from "@xrf/ipc/types/xrf-spawn";
import { GridColDef, GridRowId } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn, textColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/tools/spawn-editor/components/editor/table/SpawnTable";

interface IGraphHeaderRow extends GraphHeader {
  id: string;
}

interface ISpawnEditorGraphHeaderTableProps extends BaseComponentProps {
  header: GraphHeader;
}

export function SpawnEditorGraphHeaderTable({
  "data-testid": dataTestId = "spawn-editor-graph-header-table",
  id,
  className,
  header,
}: ISpawnEditorGraphHeaderTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("guid", "Guid", 260),
      textColumn("version", "Version", 100),
      textColumn("levelsCount", "Levels", 110),
      textColumn("verticesCount", "Vertices", 110),
      textColumn("edgesCount", "Edges", 110),
      textColumn("pointsCount", "Points", 110),
    ],
    []
  );

  const rows: Array<IGraphHeaderRow> = useMemo(() => [{ ...header, id: header.guid }], [header]);

  return (
    <SpawnTable<IGraphHeaderRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"header"}
      emptyLabel={"This graph has no header."}
      source={"Graph header"}
      getRowId={(row: IGraphHeaderRow): GridRowId => row.id}
    />
  );
}
