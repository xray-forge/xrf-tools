import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { VisualMotionNames } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionNames";
import { VisualMotionRow } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionRow";
import { VisualPanel } from "@/applications/visuals-explorer/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/applications/visuals-explorer/components/panels/VisualPanelEmpty";
import { VisualPanelSection } from "@/applications/visuals-explorer/components/panels/VisualPanelSection";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualDescription, VisualMotionDependency } from "@/core/bindings/types/xrf-visual";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * What this visual can animate from, resolved but not playable.
 *
 * A referenced omf file is looked for in the same order the model's textures are, so a missing animation set is
 * reported the same way a missing texture is. Playback is a later phase.
 */
export interface IVisualMotionsPanelProps extends BaseComponentProps {}

export function VisualMotionsPanel({
  "data-testid": dataTestId = "visual-motions-panel",
  id,
  className,
}: IVisualMotionsPanelProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const description: Nullable<VisualDescription> = visualsService.visual.value?.selected.description ?? null;

  const refs: Array<VisualMotionDependency> = visualsService.visual.value?.selected.dependencies.motions ?? [];
  const embedded: Array<string> = description?.embeddedMotions ?? [];

  if (refs.length === 0 && embedded.length === 0) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <VisualPanelEmpty label={"No motions. Resolved from the visual's omf motion refs."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
      {refs.length > 0 ? (
        <VisualPanelSection title={`Motion refs (${refs.length})`} caption={"Omf files the engine loads"} isFirst>
          {refs.map((motion: VisualMotionDependency) => (
            <VisualMotionRow key={motion.reference} motion={motion} />
          ))}
        </VisualPanelSection>
      ) : null}

      {embedded.length > 0 ? (
        <VisualPanelSection
          title={`Embedded motions (${embedded.length})`}
          caption={"Stored inside this visual"}
          isFirst={refs.length === 0}
        >
          <VisualMotionNames names={embedded} />
        </VisualPanelSection>
      ) : null}
    </VisualPanel>
  );
}
