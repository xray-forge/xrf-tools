import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { listLevelProblems } from "@/core/level/lib/problems";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { ILevelTextureReport } from "@/core/level/lib/surface/level-surface-dressing";
import { LevelLoadService } from "@/core/level/services";
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
  const service: LevelLoadService = useInjection(LevelLoadService);

  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = service.level.value?.selected.value.surfaces;
  const sectors: ReadonlyMap<number, ILoadedSector> = service.sectors;
  const report: ILevelTextureReport = service.textureReport;

  const problems: Array<IEditorProblem> = useMemo(
    () => listLevelProblems(report.problems, surfaces ?? [], sectors),
    [report, surfaces, sectors]
  );

  if (!service.level.value) {
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
