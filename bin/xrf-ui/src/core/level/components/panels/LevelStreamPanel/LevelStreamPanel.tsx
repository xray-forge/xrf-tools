import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { LevelStreamFrameSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamFrameSection";
import { LevelStreamLightsSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamLightsSection";
import { LevelStreamResidentSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamResidentSection";
import { LevelStreamStaticDrawsSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamStaticDrawsSection";
import { LevelStreamTexturesSection } from "@/core/level/components/panels/LevelStreamPanel/LevelStreamTexturesSection";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelRenderService, LevelViewportService } from "@/core/level/services";
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
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const frame: RenderFrameReport = renderService.frame;
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
      <LevelStreamResidentSection load={viewportService.load} memory={frame.memory} />

      <LevelStreamFrameSection frame={frame} />
      <LevelStreamStaticDrawsSection staticDraws={frame.staticDraws} />
      <LevelStreamLightsSection lights={frame.lights} />

      <LevelStreamTexturesSection textures={textures} />
    </EditorPanel>
  );
}
