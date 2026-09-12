import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { AlifeObject } from "@/core/bindings/types/xrf-db";
import { flagsColumn, identifierColumn, textColumn, vectorColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IAlifeObjectRow extends AlifeObject {
  index: number;
  type: string;
}

/**
 * Columns the file carries but nobody scans a table by: available in the columns panel, off by default
 * so the twenty-three column grid opens on the ones that identify an object.
 */
const HIDDEN_COLUMNS: Array<string> = [
  "scriptVersion",
  "version",
  "scriptGameId",
  "scriptRp",
  "respawnTime",
  "phantomId",
  "netAction",
  "clientDataSize",
  "direction",
];

interface ISpawnEditorAlifeObjectsTableProps extends BaseComponentProps {
  objects: Array<AlifeObject>;
}

export function SpawnEditorAlifeObjectsTable({
  "data-testid": dataTestId = "spawn-editor-alife-objects-table",
  id,
  className,
  objects,
}: ISpawnEditorAlifeObjectsTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      textColumn("index", "#", 70),
      identifierColumn("name", "Name", 220),
      identifierColumn("section", "Section", 180),
      identifierColumn("type", "Type", 200),
      identifierColumn("clsid", "Clsid", 120),
      textColumn("id", "Id", 90),
      textColumn("parentId", "Parent", 90),
      vectorColumn("position", "Position"),
      textColumn("gameType", "Game type", 110),
      flagsColumn("scriptFlags", "Script flags"),
      textColumn("scriptVersion", "Script version", 130),
      textColumn("version", "Version", 100),
      textColumn("scriptGameId", "Script game id", 130),
      textColumn("scriptRp", "Script rp", 110),
      textColumn("respawnTime", "Respawn time", 130),
      textColumn("phantomId", "Phantom", 100),
      textColumn("netAction", "Net action", 110),
      textColumn("clientDataSize", "Client data", 110),
      vectorColumn("direction", "Direction"),
    ],
    []
  );

  const rows: Array<IAlifeObjectRow> = useMemo(
    () => objects.map((it: AlifeObject, index: number) => ({ ...it, index, type: it.inherited.type })),
    [objects]
  );

  return (
    <SpawnTable<IAlifeObjectRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      countNoun={"object"}
      emptyLabel={"This file spawns no alife objects."}
      hiddenColumns={HIDDEN_COLUMNS}
      rows={rows}
      source={"Alife object"}
      getRowId={(row: IAlifeObjectRow): GridRowId => row.index}
      getSearchText={(row: IAlifeObjectRow): string => `${row.name} ${row.section} ${row.type} ${row.clsid}`}
    />
  );
}
