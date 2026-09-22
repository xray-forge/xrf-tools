import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useEffect, useState } from "react";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { VisualMotionDependency } from "@/core/ipc/types/xrf-visual";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanel, EditorPanelEmpty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { VisualMotionList } from "./VisualMotionList";
import { VisualMotionNames } from "./VisualMotionNames";
import { VisualMotionRow } from "./VisualMotionRow";
import { VisualMotionTransport } from "./VisualMotionTransport";

/**
 * What this visual animates from, and playing it.
 */
export function VisualMotionsPanel({
  "data-testid": dataTestId = "visual-motions-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const motionService: VisualMotionService = useInjection(VisualMotionService);

  const [filter, setFilter] = useState<string>("");

  const selected: Nullable<SelectedVisualDescription> = visualsService.selected;
  const refs: Array<VisualMotionDependency> = selected?.dependencies.motions ?? [];
  const embedded: Array<string> = selected?.description.embeddedMotions ?? [];
  const hasMotions: boolean = visualsService.hasMotions;
  const playable: number = motionService.motions.value?.length ?? 0;

  // Listed when this panel is on screen rather than when a model lands: naming motions means reading every animation
  // file the visual references, about fifty milliseconds each, and most models are opened to be looked at. Depending
  // on the selection is what lists the next model's motions without this panel having to unmount.
  useEffect(() => {
    if (hasMotions) {
      void motionService.list();
    }
  }, [hasMotions, motionService, selected]);

  if (!hasMotions) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <EditorPanelEmpty label={"No motions. Resolved from the visual's omf motion refs."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={cn("h-full", className)} title={"Motions"}>
      <div className={"flex h-full min-h-0 flex-col"}>
        <div className={"sticky top-0 z-1 shrink-0 border-b border-divider surface-frame px-4 pt-2 pb-3"}>
          <VisualMotionTransport />

          <div className={"mt-2"}>
            <EditorFilterInput
              query={filter}
              placeholder={"Filter motions"}
              ariaLabel={"Filter motions"}
              onQueryChange={setFilter}
            />
          </div>
        </div>

        <EditorPanelSection
          title={playable ? `Playable (${playable})` : "Playable"}
          caption={"Grouped by name prefix; double click to pose"}
          isFirst={true}
          isFilling={true}
        >
          <VisualMotionList filter={filter} />
        </EditorPanelSection>

        <div className={"max-h-1/4 shrink-0 overflow-y-auto"}>
          {refs.length > 0 ? (
            <EditorPanelSection title={`Motion refs (${refs.length})`} caption={"Omf files the engine loads"}>
              {refs.map((motion: VisualMotionDependency) => (
                <VisualMotionRow key={motion.reference} motion={motion} />
              ))}
            </EditorPanelSection>
          ) : null}

          {embedded.length > 0 ? (
            <EditorPanelSection title={`Embedded motions (${embedded.length})`} caption={"Stored inside this visual"}>
              <VisualMotionNames names={embedded} />
            </EditorPanelSection>
          ) : null}
        </div>
      </div>
    </EditorPanel>
  );
}
