import { Box, TextField } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { VisualMotionList } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionList";
import { VisualMotionNames } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionNames";
import { VisualMotionRow } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionRow";
import { VisualMotionTransport } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionTransport";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { VisualMotionDependency } from "@/core/bindings/types/xrf-visual";
import { VisualPanel, VisualPanelEmpty, VisualPanelSection } from "@/core/visuals/components/panels";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualMotionsPanelProps extends BaseComponentProps {}

/**
 * What this visual animates from, and playing it.
 */
export function VisualMotionsPanel({
  "data-testid": dataTestId = "visual-motions-panel",
  id,
  className,
}: IVisualMotionsPanelProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const motionService: VisualMotionService = useInjection(VisualMotionService);

  const [filter, setFilter] = useState<string>("");

  const selected: Nullable<SelectedVisualDescription> = visualsService.selected;
  const refs: Array<VisualMotionDependency> = selected?.dependencies.motions ?? [];
  const embedded: Array<string> = selected?.description.embeddedMotions ?? [];
  const hasMotions: boolean = visualsService.hasMotions;
  const playable: number = motionService.motions.value?.length ?? 0;

  const onFilter = useCallback((event: ChangeEvent<HTMLInputElement>) => setFilter(event.target.value), []);

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
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"}>
        <VisualPanelEmpty label={"No motions. Resolved from the visual's omf motion refs."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Motions"} sx={{ height: "100%" }}>
      <Box sx={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
        <Box
          sx={{
            position: "sticky",
            top: 0,
            zIndex: 1,
            flexShrink: 0,
            paddingX: 2,
            paddingTop: 1,
            paddingBottom: 1.5,
            backgroundColor: "background.default",
            borderBottom: 1,
            borderColor: "divider",
          }}
        >
          <VisualMotionTransport />

          <TextField
            fullWidth
            size={"small"}
            value={filter}
            placeholder={"Filter motions"}
            sx={{ marginTop: 1 }}
            slotProps={{ htmlInput: { "aria-label": "Filter motions" } }}
            onChange={onFilter}
          />
        </Box>

        <VisualPanelSection
          title={playable ? `Playable (${playable})` : "Playable"}
          caption={"Grouped by name prefix; double click to pose"}
          isFirst={true}
          isFilling={true}
        >
          <VisualMotionList filter={filter} />
        </VisualPanelSection>

        <Box sx={{ flexShrink: 0, maxHeight: "25%", overflowY: "auto" }}>
          {refs.length > 0 ? (
            <VisualPanelSection title={`Motion refs (${refs.length})`} caption={"Omf files the engine loads"}>
              {refs.map((motion: VisualMotionDependency) => (
                <VisualMotionRow key={motion.reference} motion={motion} />
              ))}
            </VisualPanelSection>
          ) : null}

          {embedded.length > 0 ? (
            <VisualPanelSection title={`Embedded motions (${embedded.length})`} caption={"Stored inside this visual"}>
              <VisualMotionNames names={embedded} />
            </VisualPanelSection>
          ) : null}
        </Box>
      </Box>
    </VisualPanel>
  );
}
