import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelStreamBudgetSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamBudgetSection";
import { LevelStreamFrameSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamFrameSection";
import { LevelStreamLightsSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamLightsSection";
import { LevelStreamReadingSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamReadingSection";
import { LevelStreamResidentSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamResidentSection";
import { LevelStreamStaticDrawsSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamStaticDrawsSection";
import { LevelStreamTexturesSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamTexturesSection";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What the viewport is holding and what it costs, measured rather than estimated.
 */
export function LevelStreamPanel({
  "data-testid": dataTestId = "level-stream-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const stats: ILevelStats = viewportService.stats;
  const textures: ILevelTextureReport = viewportService.textureReport;

  if (!loadService.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Streaming"}>
        <EditorPanelEmpty label={"No level open. Open one to see what it costs to draw."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Streaming"}>
      <LevelStreamResidentSection stats={stats} />

      <LevelStreamReadingSection stream={loadService.streamProfile} />
      <LevelStreamFrameSection stats={stats} />
      <LevelStreamStaticDrawsSection staticDraws={stats.staticDraws} />
      <LevelStreamLightsSection lights={stats.lights} />

      <LevelStreamTexturesSection textures={textures} />

      <LevelStreamBudgetSection bytes={stats.bytes} residency={loadService.residency} />
    </EditorPanel>
  );
}
