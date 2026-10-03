import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { ReactElement } from "react";

import { ERenderDebugView } from "@/core/ipc/types/xrf-renderer";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the viewport can show instead of its frame, in the order the targets are built. */
const DEBUG_VIEW_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERenderDebugView>> = [
  { label: "Final frame", value: ERenderDebugView.FINAL },
  { label: "Albedo", value: ERenderDebugView.ALBEDO },
  { label: "Gloss", value: ERenderDebugView.GLOSS },
  { label: "Normal", value: ERenderDebugView.NORMAL },
  { label: "Baked hemisphere", value: ERenderDebugView.HEMI },
  { label: "Baked sun", value: ERenderDebugView.SUN },
  { label: "Material", value: ERenderDebugView.MATERIAL },
  { label: "Depth", value: ERenderDebugView.DEPTH },
  { label: "Accumulated light", value: ERenderDebugView.LIGHT },
  { label: "Ambient occlusion", value: ERenderDebugView.AMBIENT_OCCLUSION },
];

interface ILevelOverlaysActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** Whether the settings time every pass on the GPU, which the frame readout then lists. */
  isGpuTimed: boolean;
  onToggle: (option: keyof ILevelViewOptions) => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
  /** Which picture the viewport shows: the frame, or one of the targets it was built from. */
  debugView: ERenderDebugView;
  onChangeDebugView: (debugView: ERenderDebugView) => void;
}

/**
 * What is laid over the level: the grid with its extent, the axes at the origin, and the readouts.
 */
export function LevelOverlaysAction({
  "data-testid": dataTestId = "level-overlays-action",
  id,
  className,
  options,
  isGpuTimed,
  onToggle,
  onChangeGpuTimed,
  debugView,
  onChangeDebugView,
}: ILevelOverlaysActionProps): ReactElement {
  const { isGridVisible, isAxesVisible, isStatsVisible } = options;
  const shown: Array<string> = [
    isGridVisible ? "grid" : null,
    isAxesVisible ? "axes" : null,
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
        label={"Readouts"}
        description={"What the frame cost, and where the camera stands"}
        isOn={isStatsVisible}
        onToggle={() => onToggle("isStatsVisible")}
      >
        <RenderPassTimingFormRow isChecked={isGpuTimed} onChange={onChangeGpuTimed} />
      </EditorPopoverGroupSection>

      <ChoiceListFormRow
        data-testid={"level-debug-view"}
        label={"Show"}
        description={"The finished frame, or one of the targets it was built from"}
        options={DEBUG_VIEW_OPTIONS}
        value={debugView}
        filterFrom={DEBUG_VIEW_OPTIONS.length + 1}
        onChange={onChangeDebugView}
      />
    </EditorPopoverGroup>
  );
}
