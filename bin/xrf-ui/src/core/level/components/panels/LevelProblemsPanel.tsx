import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { listLevelProblems } from "@/core/level/lib/problems";
import { ILevelSectorReport } from "@/core/level/lib/sector/level-sector-report";
import { ILevelTextureReport } from "@/core/level/lib/surface/level-surface-dressing";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { EditorProblemsPanel, IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Maybe } from "@/lib/types/general";

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
  const sectors: ILevelSectorReport = loadService.sectorReport;
  const report: ILevelTextureReport = viewportService.textureReport;

  const problems: Array<IEditorProblem> = useMemo(
    () => listLevelProblems(report.problems, surfaces ?? [], sectors.skipped),
    [report, surfaces, sectors]
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
        "Every texture this level names was read, every shader table entry was described, and every drawable of the " +
        "resident sectors was packed."
      }
    />
  );
}
