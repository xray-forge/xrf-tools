import { useInjection } from "@wirestate/react";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { describeLevelResidentWait } from "@/applications/level-viewer/lib/load/level-resident-wait";
import { listLevelProblems } from "@/applications/level-viewer/lib/problems/level-problems";
import { EMPTY_LEVEL_RESIDENT, ILevelResident } from "@/core/level/lib/render/level-resident";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
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
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const surfaces: Nullable<ReadonlyArray<XraySurfaceDescriptor>> = loadService.surfaces.value;
  const spawn: ILevelSpawnReport = loadService.spawnReport;
  const resident: ILevelResident = renderService.resident.value ?? EMPTY_LEVEL_RESIDENT;

  const problems: Array<IEditorProblem> = useMemo(
    () =>
      listLevelProblems({
        models: resident.problems.models,
        sectors: resident.problems.sectors,
        skipped: resident.problems.skipped,
        spawn,
        surfaces: surfaces ?? [],
        textures: resident.textures.problems,
      }),
    [resident, surfaces, spawn]
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
      status={describeLevelResidentWait(renderService.resident.state, "what it could not draw")}
      emptyDescription={
        "Everything this level names was read: its textures, shader entries, sectors' drawables and spawned visuals."
      }
    />
  );
}
