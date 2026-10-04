import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { LevelRendererAppliedSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererAppliedSection";
import { LevelRendererFrameSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererFrameSection";
import { LevelRendererLightsSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererLightsSection";
import { LevelRendererParticlesSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererParticlesSection";
import { LevelRendererResidentSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererResidentSection";
import { LevelRendererStaticDrawsSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererStaticDrawsSection";
import { LevelRendererTexturesSection } from "@/core/level/components/panels/LevelRendererPanel/LevelRendererTexturesSection";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelRenderService, LevelViewportService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What the viewport is holding and what it costs, measured rather than estimated.
 */
export function LevelRendererPanel({
  "data-testid": dataTestId = "level-renderer-panel",
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
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Renderer"}>
        <EditorPanelEmpty label={"No level open. Open one to see what it costs to draw."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Renderer"}>
      <LevelRendererResidentSection load={viewportService.load} memory={frame.memory} />
      <LevelRendererAppliedSection applied={renderService.applied} requested={renderService.viewOptions} />

      <LevelRendererFrameSection frame={frame} />
      <LevelRendererStaticDrawsSection staticDraws={frame.staticDraws} />
      <LevelRendererLightsSection lights={frame.lights} />
      <LevelRendererParticlesSection particles={frame.particles} />

      <LevelRendererTexturesSection textures={textures} />
    </EditorPanel>
  );
}
