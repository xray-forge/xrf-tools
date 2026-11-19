import { formatBytes, formatCount, formatDuration } from "@xrf/format";
import { LevelBuildResult } from "@xrf/ipc/types/xrf-app-levels";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LevelCompileStepReport } from "@xrf/ipc/types/xrf-level-compiler";
import { ELevelExportedPrevious, LevelBuildReport, LevelExportedFile } from "@xrf/ipc/types/xrf-rawdata";
import { Nullable } from "@xrf/types";
import { CommandResult, CommandResultPathList, ICommandResultStat } from "@xrf/ui/command-result";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { LevelRefusalFindings } from "@/core/level/components/refusals/LevelRefusalFindings";
import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { describeBuildHeadline } from "@/tools/level-compiler/lib/describe-build-headline";

import { LevelBuildSpawnAction } from "./LevelBuildSpawnAction";

const PREVIOUS_LABELS: Record<ELevelExportedPrevious, string> = {
  [ELevelExportedPrevious.NEW]: "new",
  [ELevelExportedPrevious.SAME]: "unchanged",
  [ELevelExportedPrevious.CHANGED]: "replaced",
};

/** How many of the files written stand to what was there before as `previous` says. */
function countFilesBy(files: ReadonlyArray<LevelExportedFile>, previous: ELevelExportedPrevious): number {
  return files.filter((file: LevelExportedFile) => file.previous === previous).length;
}

interface ILevelBuildResultViewProps extends BaseComponentProps {
  result: LevelBuildResult;
  /** Where the run was told to write, to reveal it; the report's paths are display text. */
  outputPath: Nullable<string>;
  /**
   * The game data the form named for the compile, handed to the spawn compiler with the level's directory only where a
   * compile ran and read it.
   */
  gamedata: Nullable<string>;
}

/**
 * What a build wrote: each file with its size and what it replaced, the files of its own it removed, the build project
 * counted, and every part refused.
 */
export function LevelBuildResultView({
  "data-testid": dataTestId = "level-build-result",
  id,
  className,
  result,
  outputPath,
  gamedata,
}: ILevelBuildResultViewProps): ReactElement {
  const report: Nullable<LevelBuildReport> = result.report;

  const stats: Array<ICommandResultStat> = useMemo(() => {
    if (!report) {
      return [];
    }

    return [
      { label: "files", value: report.files.length },
      { label: "new", value: countFilesBy(report.files, ELevelExportedPrevious.NEW) },
      { label: "replaced", value: countFilesBy(report.files, ELevelExportedPrevious.CHANGED) },
      { label: "removed", value: report.removed.length },
      { label: "faces", value: formatCount(report.geometry?.faces ?? 0) },
      { label: "lights", value: formatCount(report.geometry?.staticLights ?? 0) },
      { label: "refused", value: report.refused.length },
      // Each compile step the run took, by how long its compiler ran.
      ...(result.compile?.steps ?? []).map((step: LevelCompileStepReport) => ({
        label: step.step,
        value: formatDuration(step.duration),
      })),
    ];
  }, [report, result.compile]);

  const isCompileStopped: boolean = result.outcome === EJobOutcome.CANCELLED;

  const files: Array<string> = useMemo(
    () => [
      ...(report?.files ?? []).map(
        (file: LevelExportedFile) => `${file.name} (${formatBytes(file.size)}, ${PREVIOUS_LABELS[file.previous]})`
      ),
      // What an earlier build wrote and this forced one does not, which it took away.
      ...(report?.removed ?? []).map((name: string) => `${name} (removed)`),
    ],
    [report]
  );

  if (!report) {
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
      headline={describeBuildHeadline(report, result.compile, isCompileStopped)}
      tone={report.refused.length > 0 || isCompileStopped ? "warning" : "success"}
      stats={stats}
      actions={
        <>
          <RevealPathButton tool={EToolId.LEVEL_COMPILER} path={outputPath} label={"Show level"} />
          <LevelBuildSpawnAction level={outputPath} gamedata={result.compile ? gamedata : null} />
        </>
      }
    >
      <CommandResultPathList
        paths={files}
        column={"File"}
        emptyLabel={"Nothing was written."}
        searchPlaceholder={"Filter by file"}
      />

      {report.refused.length > 0 ? (
        <LevelRefusalFindings refused={report.refused} emptyLabel={"Every part was built."} />
      ) : null}
    </CommandResult>
  );
}
