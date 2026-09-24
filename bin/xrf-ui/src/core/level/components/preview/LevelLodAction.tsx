import { default as ForestIcon } from "@mui/icons-material/Forest";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { DEFAULT_LEVEL_LOD_OPTIONS, ILevelLodOptions, LEVEL_LOD_LIMITS } from "@/core/level/lib/lod/level-lod-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

interface ILevelLodActionProps extends BaseComponentProps {
  isOn: boolean;
  lod: ILevelLodOptions;
  onToggle: () => void;
  onChange: (lod: ILevelLodOptions) => void;
}

/**
 * Whether distant clumps of trees are drawn as their impostors, and how far away they turn.
 */
export function LevelLodAction({
  "data-testid": dataTestId = "level-lod-action",
  id,
  className,
  isOn,
  lod,
  onToggle,
  onChange,
}: ILevelLodActionProps): ReactElement {
  const distance: string = `${formatNumber(lod.distance, 2)}×`;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Impostors"}
      description={isOn ? `Impostors past ${distance} the game's distance` : "Impostors off, every tree drawn"}
      icon={<ForestIcon />}
      isOn={isOn}
      toggleLabel={"Draw distant trees as impostors"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Distance"}
        value={lod.distance}
        {...LEVEL_LOD_LIMITS.distance}
        format={(value: number) => `${formatNumber(value, 2)}×`}
        onChange={(value: number) => onChange({ ...lod, distance: value })}
      />

      <Button size={"small"} onClick={() => onChange(DEFAULT_LEVEL_LOD_OPTIONS)}>
        Back to the game&apos;s distance
      </Button>
    </EditorPopoverToggle>
  );
}
