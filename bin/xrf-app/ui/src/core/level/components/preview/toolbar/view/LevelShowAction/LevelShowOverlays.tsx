import { useInjection } from "@wirestate/react";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Fragment, ReactElement } from "react";

import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
import { LevelViewService } from "@/core/level/services/level-view";
import {
  RENDER_PASS_TIMING_DESCRIPTION,
  RENDER_PASS_TIMING_LABEL,
} from "@/core/render/components/controls/RenderPassTimingFormRow";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/**
 * What is laid over the level, a switch each: the grid with its extent, the axes, the sun's direction, the readouts
 * with whether the settings time every pass for them, the spot meter and the calibration props.
 */
export function LevelShowOverlays({
  "data-testid": dataTestId = "level-show-overlays",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  return (
    <EditorPopoverGroupSection data-testid={dataTestId} id={id} className={className} label={"Overlays"}>
      {LEVEL_OVERLAYS.map((overlay: ILevelOverlay) => (
        <Fragment key={overlay.id}>
          <EditorPopoverSwitchRow
            label={overlay.label}
            description={overlay.description}
            isChecked={viewService.options[overlay.option]}
            onChange={() => viewService.toggleOption(overlay.option)}
          />

          {overlay.option === "isStatsVisible" ? (
            <EditorPopoverSwitchRow
              label={RENDER_PASS_TIMING_LABEL}
              description={RENDER_PASS_TIMING_DESCRIPTION}
              isChecked={renderSettingsService.isGpuTimed}
              isNested
              onChange={renderSettingsService.setGpuTimed}
            />
          ) : null}
        </Fragment>
      ))}
    </EditorPopoverGroupSection>
  );
}
