import { useInjection } from "@wirestate/react";
import { Maybe } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { WorldLevelProblems } from "@/core/ipc/types/xrf-world";
import { listLevelProblems } from "@/core/level/lib/problems";
import { ILevelSpawnReport } from "@/core/level/lib/spawn";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorProblemsPanel, IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Everything the viewer could not draw the way the level asked for it.
 */
export function LevelProblemsPanel({
  "data-testid": dataTestId = "level-problems-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = loadService.level.value?.selected.value.surfaces;
  const spawn: ILevelSpawnReport = loadService.spawnReport;
  const report: ILevelTextureReport = viewportService.textureReport;
  const native: WorldLevelProblems = viewportService.problems;

  const problems: Array<IEditorProblem> = useMemo(
    () =>
      listLevelProblems({
        models: native.models,
        sectors: native.sectors,
        skipped: native.skipped,
        spawn,
        surfaces: surfaces ?? [],
        textures: report.problems,
      }),
    [report, surfaces, native, spawn]
  );

  if (!loadService.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Problems"}>
        <EditorPanelEmpty label={"No level open. Open one to see what it could not be drawn from."} />
      </EditorPanel>
    );
  }

  return (
    <EditorProblemsPanel
      data-testid={dataTestId}
      id={id}
      className={className}
      findings={problems}
      emptyDescription={
        "Every texture this level names was read, every shader table entry was described, every drawable of the " +
        "resident sectors was packed, and every visual its spawned objects name was read."
      }
    />
  );
}
