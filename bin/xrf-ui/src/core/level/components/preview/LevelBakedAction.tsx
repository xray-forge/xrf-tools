import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelBakedActionProps extends BaseComponentProps {
  isOn: boolean;
  lighting: ILevelLighting;
  onToggle: () => void;
  onChange: (lighting: ILevelLighting) => void;
}

/**
 * Whether the hemisphere occlusion xrLC baked into the level darkens its ambient, and by how much.
 */
export function LevelBakedAction({
  "data-testid": dataTestId = "level-baked-action",
  id,
  className,
  isOn,
  lighting,
  onToggle,
  onChange,
}: ILevelBakedActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Baked light"}
      description={isOn ? `Baked occlusion at ${formatPercent(lighting.hemiStrength)}` : "Baked occlusion off"}
      icon={<LightbulbIcon />}
      isOn={isOn}
      toggleLabel={"Apply the baked occlusion"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Occlusion"}
        value={lighting.hemiStrength}
        min={0}
        max={1}
        step={0.05}
        format={formatPercent}
        onChange={(hemiStrength: number) => onChange({ ...lighting, hemiStrength })}
      />

      <Button
        size={"small"}
        onClick={() => onChange({ ...lighting, hemiStrength: DEFAULT_LEVEL_LIGHTING.hemiStrength })}
      >
        Back to the whole occlusion
      </Button>
    </EditorPopoverToggle>
  );
}
