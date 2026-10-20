import { useInjection } from "@wirestate/react";
import { Maybe } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { describeLevelDescriptionWait } from "@/applications/level-viewer/lib/load/level-description-wait";
import { listLevelProblems } from "@/applications/level-viewer/lib/problems/level-problems";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { WorldLevelProblems } from "@/core/ipc/types/xrf-world";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorProblemsPanel, IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Everything the viewer could not draw the way the level asked for it; what the renderer could not draw is said once
 * the level is resident, and the panel says so until then rather than calling the level clean.
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
      status={describeLevelDescriptionWait(viewportService.description, "what it could not draw")}
      emptyDescription={
        "Everything this level names was read: its textures, shader entries, sectors' drawables and spawned visuals."
      }
    />
  );
}
