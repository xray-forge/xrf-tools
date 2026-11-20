import { formatCount } from "@xrf/format";
import { SpawnBuildResult } from "@xrf/ipc/types/xrf-app-spawn";
import { Finding } from "@xrf/ipc/types/xrf-report";
import { SpawnRootOutcome } from "@xrf/ipc/types/xrf-spawn-builder";
import { Nullable } from "@xrf/types";
import { CommandResult, CommandResultFindings, ICommandResultStat } from "@xrf/ui/command-result";
import { GridColDef } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { listSpawnFindings } from "@/tools/spawn-compiler/lib/spawn-findings";

interface ISpawnBuildResultViewProps extends BaseComponentProps {
  result: SpawnBuildResult;
  /** The directory the run was told to write into, to reveal it; the outcome's path is display text. */
  outputPath: Nullable<string>;
}

/**
 * What a spawn's build wrote: the spawn and the levels it holds, what it counts, and every finding its report holds,
 * each naming its level.
 */
export function SpawnBuildResultView({
  "data-testid": dataTestId = "spawn-build-result",
  id,
  className,
  result,
  outputPath,
}: ISpawnBuildResultViewProps): ReactElement {
  const spawn: Nullable<SpawnRootOutcome> = result.spawn;

  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("subject", "Level", 160),
      identifierColumn("ruleId", "Rule", 200),
      { field: "message", headerName: "Detail", flex: 2, minWidth: 360 },
    ],
    []
  );

  const rows: Array<Finding & { id: number }> = useMemo(
    () => (spawn ? listSpawnFindings(spawn).map((finding: Finding, index: number) => ({ ...finding, id: index })) : []),
    [spawn]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () =>
      spawn
        ? [
            { label: "levels", value: spawn.levels.length },
            { label: "objects", value: formatCount(spawn.objects) },
            { label: "artefact points", value: formatCount(spawn.artefactPoints) },
            { label: "patrols", value: formatCount(spawn.patrols) },
            { label: "findings", value: rows.length },
          ]
        : [],
    [rows.length, spawn]
  );

  if (!spawn) {
    return (
      <CommandResult
        data-testid={dataTestId}
        id={id}
        className={className}
        headline={"Stopped before writing; nothing was written"}
        tone={"warning"}
        stats={[]}
      />
    );
  }

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={`Built ${spawn.path} from ${spawn.levels.join(", ")}`}
      tone={rows.length > 0 ? "warning" : "success"}
      stats={stats}
      actions={<RevealPathButton tool={EToolId.SPAWN_COMPILER} path={outputPath} label={"Show spawn"} />}
    >
      <CommandResultFindings<Finding & { id: number }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.id}
        getSearchText={(row) => `${row.subject ?? ""} ${row.ruleId} ${row.message}`}
        emptyLabel={"Every graph point, connection and restrictor went in as the levels hold them."}
        searchPlaceholder={"Filter findings"}
      />
    </CommandResult>
  );
}
