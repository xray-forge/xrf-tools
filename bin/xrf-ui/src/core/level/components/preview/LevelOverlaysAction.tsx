import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelViewService } from "@/core/level/services/level-view.service";
import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What is laid over the level: the grid with its extent, the axes at the origin, the sun's direction, and the readouts;
 * with the readouts, whether the settings time every pass on the GPU, which the frame readout then lists.
 */
export function LevelOverlaysAction({
  "data-testid": dataTestId = "level-overlays-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const { isGridVisible, isAxesVisible, isSunMarked, isStatsVisible } = viewService.options;
  const isGpuTimed: boolean = renderSettingsService.isGpuTimed;
  const shown: Array<string> = [
    isGridVisible ? "grid" : null,
    isAxesVisible ? "axes" : null,
    isSunMarked ? "sun" : null,
    isStatsVisible ? "readouts" : null,
  ].filter((it): it is string => it !== null);

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Overlays"}
      description={shown.length ? `Showing ${shown.join(", ")}` : "No overlays, a clean look at the level"}
      icon={<GridOnIcon />}
      isActive={shown.length > 0}
    >
      <EditorPopoverGroupSection
        label={"Grid"}
        description={"The ground plane in round cells, and the extent the level claims"}
        isOn={isGridVisible}
        onToggle={() => viewService.toggleOption("isGridVisible")}
      />

      <EditorPopoverGroupSection
        label={"Axes"}
        description={"At the level's origin, saying which way +x and +z go"}
        isOn={isAxesVisible}
        onToggle={() => viewService.toggleOption("isAxesVisible")}
      />

      <EditorPopoverGroupSection
        label={"Sun"}
        description={"A dot in the sky where the level's directional light comes from"}
        isOn={isSunMarked}
        onToggle={() => viewService.toggleOption("isSunMarked")}
      />

      <EditorPopoverGroupSection
        label={"Readouts"}
        description={"What the frame cost, and where the camera stands"}
        isOn={isStatsVisible}
        onToggle={() => viewService.toggleOption("isStatsVisible")}
      >
        <RenderPassTimingFormRow isChecked={isGpuTimed} onChange={renderSettingsService.setGpuTimed} />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
