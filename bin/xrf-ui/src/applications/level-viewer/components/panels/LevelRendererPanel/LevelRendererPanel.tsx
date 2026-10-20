import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelRendererAppliedSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererAppliedSection";
import { LevelRendererFrameSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererFrameSection";
import { LevelRendererGraphSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererGraphSection";
import { LevelRendererGrassSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererGrassSection";
import { LevelRendererLightsSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererLightsSection";
import { LevelRendererMemorySection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererMemorySection";
import { LevelRendererParticlesSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererParticlesSection";
import { LevelRendererResidentSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererResidentSection";
import { LevelRendererStaticDrawsSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererStaticDrawsSection";
import { LevelRendererTexturesSection } from "@/applications/level-viewer/components/panels/LevelRendererPanel/LevelRendererTexturesSection";
import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { RenderSettingsService } from "@/core/render/services/render-settings";
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
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

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
      <LevelRendererMemorySection memory={frame.memory} />
      <LevelRendererAppliedSection applied={renderService.applied} requested={renderService.viewOptions} />

      <LevelRendererFrameSection frame={frame} />
      <LevelRendererGraphSection
        graph={frame.graph}
        settings={renderSettingsService.graph}
        onChange={renderSettingsService.setGraph}
      />
      <LevelRendererStaticDrawsSection staticDraws={frame.staticDraws} />
      <LevelRendererLightsSection lights={frame.lights} />
      <LevelRendererParticlesSection particles={frame.particles} />
      <LevelRendererGrassSection grass={frame.grass} />

      <LevelRendererTexturesSection textures={textures} />
    </EditorPanel>
  );
}
