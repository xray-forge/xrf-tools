import { formatCount } from "@xrf/format";
import { LevelDecompileResult } from "@xrf/ipc/types/xrf-app-levels";
import { LevelDecompileReport } from "@xrf/ipc/types/xrf-rawdata";
import { Nullable } from "@xrf/types";
import { CommandResult, ICommandResultStat } from "@xrf/ui/command-result";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { LevelRefusalFindings } from "@/core/level/components/refusals/LevelRefusalFindings";
import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";

interface ILevelDecompileResultViewProps extends BaseComponentProps {
  result: LevelDecompileResult;
  /** Where the run was told to write the scene, to reveal it; the report's paths are display text. */
  outputPath: Nullable<string>;
}

/**
 * What a decompile wrote: the scene's tools, the static geometry counted, and every piece the scene could not hold.
 */
export function LevelDecompileResultView({
  "data-testid": dataTestId = "level-decompile-result",
  id,
  className,
  result,
  outputPath,
}: ILevelDecompileResultViewProps): ReactElement {
  const report: Nullable<LevelDecompileReport> = result.report;

  const stats: Array<ICommandResultStat> = useMemo(() => {
    if (!report) {
      return [];
    }

    return [
      { label: "tools", value: report.tools.length },
      { label: "objects", value: formatCount(report.geometry?.objects ?? 0) },
      { label: "faces", value: formatCount(report.geometry?.faces ?? 0) },
      { label: "tree models", value: formatCount(report.geometry?.treeModels ?? 0) },
      {
        label: "library objects",
        value: formatCount((report.geometry?.instances ?? 0) + (report.geometry?.libraryTreeModels ?? 0)),
      },
      { label: "textures", value: report.textures },
      { label: "inferred", value: report.inferred.length },
      { label: "left out", value: report.refused.length },
    ];
  }, [report]);

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
      headline={`Decompiled ${report.source} into ${report.destination}`}
      tone={report.refused.length > 0 ? "warning" : "success"}
      stats={stats}
      actions={<RevealPathButton tool={EToolId.LEVEL_DECOMPILER} path={outputPath} label={"Show scene"} />}
    >
      <LevelRefusalFindings refused={report.refused} emptyLabel={"The scene holds everything the level did."} />
    </CommandResult>
  );
}
