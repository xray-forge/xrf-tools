import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { DEFAULT_LEVEL_HEMI_STRENGTH } from "@/core/level/lib/view/level-view-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelBakedActionProps extends BaseComponentProps {
  isOn: boolean;
  /** How much the baked hemisphere darkens the ambient. */
  hemiStrength: number;
  onToggle: () => void;
  onChange: (hemiStrength: number) => void;
}

/**
 * Whether the hemisphere occlusion xrLC baked into the level darkens its ambient, and by how much.
 */
export function LevelBakedAction({
  "data-testid": dataTestId = "level-baked-action",
  id,
  className,
  isOn,
  hemiStrength,
  onToggle,
  onChange,
}: ILevelBakedActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Baked light"}
      description={isOn ? `Baked occlusion at ${formatPercent(hemiStrength)}` : "Baked occlusion off"}
      icon={<LightbulbIcon />}
      isOn={isOn}
      toggleLabel={"Apply the baked occlusion"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Occlusion"}
        value={hemiStrength}
        min={0}
        max={1}
        step={0.05}
        format={formatPercent}
        onChange={onChange}
      />

      <Button size={"small"} onClick={() => onChange(DEFAULT_LEVEL_HEMI_STRENGTH)}>
        Back to the whole occlusion
      </Button>
    </EditorPopoverToggle>
  );
}
