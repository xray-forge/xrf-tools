import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { ReactElement } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelOverlaysActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** Whether the settings time every pass on the GPU, which the frame readout then lists. */
  isGpuTimed: boolean;
  onToggle: (option: keyof ILevelViewOptions) => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
}

/**
 * What is laid over the level: the grid with its extent, the axes at the origin, the sun's direction, and the readouts.
 */
export function LevelOverlaysAction({
  "data-testid": dataTestId = "level-overlays-action",
  id,
  className,
  options,
  isGpuTimed,
  onToggle,
  onChangeGpuTimed,
}: ILevelOverlaysActionProps): ReactElement {
  const { isGridVisible, isAxesVisible, isSunMarked, isStatsVisible } = options;
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
        onToggle={() => onToggle("isGridVisible")}
      />

      <EditorPopoverGroupSection
        label={"Axes"}
        description={"At the level's origin, saying which way +x and +z go"}
        isOn={isAxesVisible}
        onToggle={() => onToggle("isAxesVisible")}
      />

      <EditorPopoverGroupSection
        label={"Sun"}
        description={"A dot in the sky where the level's directional light comes from"}
        isOn={isSunMarked}
        onToggle={() => onToggle("isSunMarked")}
      />

      <EditorPopoverGroupSection
        label={"Readouts"}
        description={"What the frame cost, and where the camera stands"}
        isOn={isStatsVisible}
        onToggle={() => onToggle("isStatsVisible")}
      >
        <RenderPassTimingFormRow isChecked={isGpuTimed} onChange={onChangeGpuTimed} />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
