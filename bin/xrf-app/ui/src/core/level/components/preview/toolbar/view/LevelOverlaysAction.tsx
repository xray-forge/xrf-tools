import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
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

  const isGpuTimed: boolean = renderSettingsService.isGpuTimed;
  const shown: Array<string> = LEVEL_OVERLAYS.filter((it: ILevelOverlay) => viewService.options[it.option]).map(
    (it: ILevelOverlay) => it.label.toLowerCase()
  );

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
      {LEVEL_OVERLAYS.map((it: ILevelOverlay) => (
        <EditorPopoverGroupSection
          key={it.id}
          label={it.label}
          description={it.description}
          isOn={viewService.options[it.option]}
          onToggle={() => viewService.toggleOption(it.option)}
        >
          {it.option === "isStatsVisible" ? (
            <RenderPassTimingFormRow isChecked={isGpuTimed} onChange={renderSettingsService.setGpuTimed} />
          ) : null}
        </EditorPopoverGroupSection>
      ))}
    </EditorPopoverGroup>
  );
}
