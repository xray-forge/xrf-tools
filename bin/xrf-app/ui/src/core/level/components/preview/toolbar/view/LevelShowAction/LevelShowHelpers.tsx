import { useInjection } from "@wirestate/react";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverSliderRow, EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import {
  ILevelHelperOptions,
  LEVEL_HELPER_DISTANCE_LIMITS,
  LEVEL_HELPER_OCCLUDED_LIMITS,
} from "@/core/level/lib/objects/level-helper-options";
import { LevelViewService } from "@/core/level/services/level-view";

/**
 * The helpers' master switch, `G`, over how they are drawn: the unmarked fade, screenshots, the wire behind the level
 * and the fade distance, each waiting on the switch.
 */
export function LevelShowHelpers({
  "data-testid": dataTestId = "level-show-helpers",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const helpers: ILevelHelperOptions = viewService.helpers;
  const isOff: boolean = !helpers.isShown;

  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Helpers"}
      description={"Draws the inspector's wires and marks over the level objects (G); off, the photo mode."}
      isOn={helpers.isShown}
      onToggle={viewService.toggleHelpers}
    >
      <EditorPopoverSwitchRow
        label={"Fade the rest"}
        description={"Dims and greys what no helper marks, so the helped objects stand out."}
        isChecked={helpers.isRestFaded}
        isDisabled={isOff}
        onChange={(isRestFaded: boolean) => viewService.setHelperLook({ isRestFaded })}
      />

      <EditorPopoverSwitchRow
        label={"Include in screenshots"}
        description={"Draws the helpers into a screenshot (F9) as the view shows them; a map's are its annotations."}
        isChecked={helpers.isScreenshotHelped}
        isDisabled={isOff}
        onChange={(isScreenshotHelped: boolean) => viewService.setHelperLook({ isScreenshotHelped })}
      />

      <EditorPopoverSliderRow
        label={"Behind level"}
        description={"How strongly a wire shows where the level stands in front of it."}
        value={helpers.occludedStrength}
        {...LEVEL_HELPER_OCCLUDED_LIMITS}
        isDisabled={isOff}
        format={(strength: number) => `${Math.round(strength * 100)}%`}
        onChange={(occludedStrength: number) => viewService.setHelperLook({ ...helpers, occludedStrength })}
      />

      <EditorPopoverSliderRow
        label={"Draw distance"}
        description={"Scales how far off each kind's helpers still draw."}
        value={helpers.distanceScale}
        {...LEVEL_HELPER_DISTANCE_LIMITS}
        isDisabled={isOff}
        format={(scale: number) => `${scale}×`}
        onChange={(distanceScale: number) => viewService.setHelperLook({ ...helpers, distanceScale })}
      />
    </EditorPopoverGroupSection>
  );
}
