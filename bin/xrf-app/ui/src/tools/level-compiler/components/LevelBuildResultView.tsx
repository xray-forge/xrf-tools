import { formatBytes, formatCount } from "@xrf/format";
import { LevelBuildResult } from "@xrf/ipc/types/xrf-app-levels";
import { ELevelExportedPrevious, LevelBuildReport, LevelExportedFile } from "@xrf/ipc/types/xrf-rawdata";
import { Nullable } from "@xrf/types";
import { CommandResult, CommandResultPathList, ICommandResultStat } from "@xrf/ui/command-result";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { LevelRefusalFindings } from "@/core/level/components/refusals/LevelRefusalFindings";
import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";

const PREVIOUS_LABELS: Record<ELevelExportedPrevious, string> = {
  [ELevelExportedPrevious.NEW]: "new",
  [ELevelExportedPrevious.SAME]: "unchanged",
  [ELevelExportedPrevious.CHANGED]: "replaced",
};

interface ILevelBuildResultViewProps extends BaseComponentProps {
  result: LevelBuildResult;
  /** Where the run was told to write, to reveal it; the report's paths are display text. */
  outputPath: Nullable<string>;
}

/**
 * What a build wrote: each file with its size and what it replaced, the build project counted, and every part refused.
 */
export function LevelBuildResultView({
  "data-testid": dataTestId = "level-build-result",
  id,
  className,
  result,
  outputPath,
}: ILevelBuildResultViewProps): ReactElement {
  const report: Nullable<LevelBuildReport> = result.report;

  const stats: Array<ICommandResultStat> = useMemo(() => {
    if (!report) {
      return [];
    }

    function count(files: Array<LevelExportedFile>, previous: ELevelExportedPrevious): number {
      return files.filter((file: LevelExportedFile) => file.previous === previous).length;
    }

    return [
      { label: "files", value: report.files.length },
      { label: "new", value: count(report.files, ELevelExportedPrevious.NEW) },
      { label: "replaced", value: count(report.files, ELevelExportedPrevious.CHANGED) },
      { label: "faces", value: formatCount(report.geometry?.faces ?? 0) },
      { label: "lights", value: formatCount(report.geometry?.staticLights ?? 0) },
      { label: "refused", value: report.refused.length },
    ];
  }, [report]);

  const files: Array<string> = useMemo(
    () =>
      (report?.files ?? []).map(
        (file: LevelExportedFile) => `${file.name} (${formatBytes(file.size)}, ${PREVIOUS_LABELS[file.previous]})`
      ),
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
      headline={`Built ${report.source} into ${report.destination}`}
      tone={report.refused.length > 0 ? "warning" : "success"}
      stats={stats}
      actions={<RevealPathButton tool={EToolId.LEVEL_COMPILER} path={outputPath} label={"Show level"} />}
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
