import { GridColDef, GridRowId } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { SpawnTable } from "@/applications/spawn-editor/components/editor/table/SpawnTable";
import { Patrol, PatrolLink } from "@/core/ipc/types/xrf-db";
import { identifierColumn, textColumn } from "@/core/ui/table";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IPatrolLinkRow {
  id: string;
  patrol: string;
  index: number;
  linksCount: number;
  /** `to(weight)` pairs, which is how a link list reads without expanding the panel. */
  links: string;
}

interface ISpawnEditorPatrolLinksTableProps extends BaseComponentProps {
  patrols: Array<Patrol>;
}

export function SpawnEditorPatrolLinksTable({
  "data-testid": dataTestId = "spawn-editor-patrol-links-table",
  id,
  className,
  patrols,
}: ISpawnEditorPatrolLinksTableProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("patrol", "Patrol", 300),
      textColumn("index", "From point", 120),
      textColumn("linksCount", "Links", 100),
      identifierColumn("links", "Targets", 260),
    ],
    []
  );

  const rows: Array<IPatrolLinkRow> = useMemo(
    () =>
      patrols.flatMap((patrol: Patrol) =>
        patrol.links.map((link: PatrolLink) => ({
          id: `${patrol.name}/${link.index}`,
          index: link.index,
          links: link.links.map(([to, weight]: [number, Nullable<number>]) => `${to}(${weight ?? 0})`).join(", "),
          linksCount: link.links.length,
          patrol: patrol.name,
        }))
      ),
    [patrols]
  );

  return (
    <SpawnTable<IPatrolLinkRow>
      data-testid={dataTestId}
      id={id}
      className={className}
      columns={columns}
      rows={rows}
      countNoun={"link"}
      emptyLabel={"These patrols have no links."}
      source={"Patrol link"}
      getRowId={(row: IPatrolLinkRow): GridRowId => row.id}
      getSearchText={(row: IPatrolLinkRow): string => row.patrol}
    />
  );
}
