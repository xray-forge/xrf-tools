import { useInjection } from "@wirestate/react";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";

/**
 * What plays in the level, a switch each: the particle systems it plants and its zones emit, whether its campfires
 * burn, and the weather's ambient effects near the camera.
 */
export function LevelShowEffects({
  "data-testid": dataTestId = "level-show-effects",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { isParticled, isCampfireLit, isAmbientEffectPlayed }: ILevelViewOptions = viewService.options;

  return (
    <EditorPopoverGroupSection data-testid={dataTestId} id={id} className={className} label={"Effects"}>
      <EditorPopoverSwitchRow
        label={"Particles"}
        description={"Plays what the level places and its zones emit: steam, fire, anomalies' idle effects."}
        isChecked={isParticled}
        onChange={() => viewService.toggleOption("isParticled")}
      />

      <EditorPopoverSwitchRow
        label={"Campfires lit"}
        description={"Burn, as the engine starts them; off, they smoulder as the game's scripts leave them."}
        isChecked={isCampfireLit}
        onChange={() => viewService.toggleOption("isCampfireLit")}
      />

      <EditorPopoverSwitchRow
        label={"Ambient effects"}
        description={"Plays the weather's fog, leaves and vortices near the camera outdoors, with their wind."}
        isChecked={isAmbientEffectPlayed}
        onChange={() => viewService.toggleOption("isAmbientEffectPlayed")}
      />
    </EditorPopoverGroupSection>
  );
}
