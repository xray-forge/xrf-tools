import { default as LocalFireDepartmentIcon } from "@mui/icons-material/LocalFireDepartment";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelViewService } from "@/core/level/services/level-view.service";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
        description={"What the level plants and its zones play: steam, fire, anomalies' idle effects"}
        isOn={isParticled}
        onToggle={() => viewService.toggleOption("isParticled")}
      />

      <EditorPopoverGroupSection
        label={"Campfires lit"}
        description={
          isCampfireLit ? "Burning, as the engine starts them" : "Smouldering out, as the game's scripts leave them"
        }
        isOn={isCampfireLit}
        onToggle={() => viewService.toggleOption("isCampfireLit")}
      />

      <EditorPopoverGroupSection
        label={"Ambient effects"}
        description={
          isAmbientPlayed
            ? "The weather's fog, leaves and vortices near the camera, outdoors, with their wind"
            : "None near the camera, and still air"
        }
        isOn={isAmbientPlayed}
        onToggle={() => viewService.toggleOption("isAmbientPlayed")}
      />
    </EditorPopoverGroup>
  );
}
