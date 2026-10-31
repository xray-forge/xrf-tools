import { useInjection } from "@wirestate/react";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ViewInArIcon } from "@xrf/ui/icons/view-in-ar";
import { ReactElement } from "react";

import {
  ILevelHelperOptions,
  LEVEL_HELPER_DISTANCE_LIMITS,
  LEVEL_HELPER_OCCLUDED_LIMITS,
} from "@/core/level/lib/objects/level-helper-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

/**
 * The helpers' master switch, `G`: the inspector's wires and marks over the level objects, or the photo mode with none,
 * each kind keeping its choice; whether what they leave unmarked fades and whether a screenshot draws them too; and how
 * the wires show behind the level and how far off they fade.
 */
export function LevelHelpersAction({
  "data-testid": dataTestId = "level-helpers-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const helpers: ILevelHelperOptions = viewService.helpers;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Helpers"}
      description={helpers.isShown ? "Helpers drawn over the level objects (G)" : "Helpers off, the photo mode (G)"}
      icon={<ViewInArIcon />}
      isOn={helpers.isShown}
      toggleLabel={"Helpers"}
      onToggle={viewService.toggleHelpers}
    >
      <CheckboxFormRow
        label={"Fade the rest"}
        description={"Dims and greys what no helper marks, so the helped objects stand out"}
        isChecked={helpers.isRestFaded}
        onChange={(isRestFaded: boolean) => viewService.setHelperLook({ isRestFaded })}
      />
      <CheckboxFormRow
        label={"Include in screenshots"}
        description={"Draws the helpers into a screenshot (F9) as the view shows them; a map's are its annotations"}
        isChecked={helpers.isScreenshotHelped}
        onChange={(isScreenshotHelped: boolean) => viewService.setHelperLook({ isScreenshotHelped })}
      />
      <RenderValueSlider
        label={"Behind the level"}
        value={helpers.occludedStrength}
        {...LEVEL_HELPER_OCCLUDED_LIMITS}
        format={(strength: number) => `${Math.round(strength * 100)}%`}
        onChange={(occludedStrength: number) => viewService.setHelperLook({ ...helpers, occludedStrength })}
      />
      <RenderValueSlider
        label={"Draw distance"}
        value={helpers.distanceScale}
        {...LEVEL_HELPER_DISTANCE_LIMITS}
        format={(scale: number) => `${scale}×`}
        onChange={(distanceScale: number) => viewService.setHelperLook({ ...helpers, distanceScale })}
      />
    </EditorPopoverToggle>
  );
}
