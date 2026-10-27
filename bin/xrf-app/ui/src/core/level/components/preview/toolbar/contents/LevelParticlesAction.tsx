import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { LocalFireDepartmentIcon } from "@xrf/ui/icons/local-fire-department";
import { ReactElement } from "react";

import { LevelViewService } from "@/core/level/services/level-view";

/**
 * Whether the particle systems the level plants and its zones play are drawn, whether its campfires burn, and whether
 * the weather's ambient effects play near the camera.
 */
export function LevelParticlesAction({
  "data-testid": dataTestId = "level-particles-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);

  const { isParticled, isCampfireLit, isAmbientPlayed } = viewService.options;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Particles"}
      description={isParticled ? `Particles playing, campfires ${isCampfireLit ? "lit" : "out"}` : "Particles off"}
      icon={<LocalFireDepartmentIcon />}
      isActive={isParticled}
    >
      <EditorPopoverGroupSection
        label={"Particles"}
        description={"Plays what the level places and its zones emit: steam, fire, anomalies' idle effects"}
        isOn={isParticled}
        onToggle={() => viewService.toggleOption("isParticled")}
      />

      <EditorPopoverGroupSection
        label={"Campfires lit"}
        description={isCampfireLit ? "Burn, as the engine starts them" : "Smoulder, as the game's scripts leave them"}
        isOn={isCampfireLit}
        onToggle={() => viewService.toggleOption("isCampfireLit")}
      />

      <EditorPopoverGroupSection
        label={"Ambient effects"}
        description={
          isAmbientPlayed
            ? "Plays the weather's fog, leaves and vortices near the camera outdoors, with their wind"
            : "Plays none, and the air stays still"
        }
        isOn={isAmbientPlayed}
        onToggle={() => viewService.toggleOption("isAmbientPlayed")}
      />
    </EditorPopoverGroup>
  );
}
