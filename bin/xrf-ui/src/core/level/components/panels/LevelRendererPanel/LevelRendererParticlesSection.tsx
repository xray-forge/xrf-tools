import { formatCount, formatMilliseconds } from "@xrf/format";
import { ReactElement } from "react";

import { RenderParticlesReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelRendererParticlesSectionProps extends BaseComponentProps {
  particles: RenderParticlesReport;
}

/**
 * What the particle systems came to: the effects playing and their particles, how many stepped and drew on the last
 * frame, and what stepping them cost.
 */
export function LevelRendererParticlesSection({
  "data-testid": dataTestId = "level-renderer-particles-section",
  id,
  className,
  particles,
}: ILevelRendererParticlesSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Particles"}>
      <EditorPanelProperty label={"Effects playing"} value={formatCount(particles.effects)} />
      <EditorPanelProperty label={"Particles"} value={formatCount(particles.particles)} />
      <EditorPanelProperty label={"Stepped"} value={formatCount(particles.simulated)} />
      <EditorPanelProperty label={"Drawn"} value={formatCount(particles.drawn)} />
      <EditorPanelProperty label={"Stepping"} value={formatMilliseconds(particles.simulationTime ?? 0)} />
    </EditorPanelSection>
  );
}
