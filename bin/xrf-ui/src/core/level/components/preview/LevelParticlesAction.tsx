import { default as LocalFireDepartmentIcon } from "@mui/icons-material/LocalFireDepartment";
import { ReactElement } from "react";

import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelParticlesActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
}

/**
 * Whether the particle systems the level plants and its zones play are drawn, and whether its campfires burn.
 */
export function LevelParticlesAction({
  "data-testid": dataTestId = "level-particles-action",
  id,
  className,
  options,
  onToggle,
}: ILevelParticlesActionProps): ReactElement {
  const { isParticled, isCampfireLit } = options;

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
        onToggle={() => onToggle("isParticled")}
      />

      <EditorPopoverGroupSection
        label={"Campfires lit"}
        description={
          isCampfireLit ? "Burning, as the engine starts them" : "Smouldering out, as the game's scripts leave them"
        }
        isOn={isCampfireLit}
        onToggle={() => onToggle("isCampfireLit")}
      />
    </EditorPopoverGroup>
  );
}
