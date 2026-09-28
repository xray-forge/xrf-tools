import { default as ForestIcon } from "@mui/icons-material/Forest";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import {
  DEFAULT_LEVEL_LOD_OPTIONS,
  formatLevelLodDistance,
  ILevelLodOptions,
  LEVEL_LOD_LIMITS,
} from "@/core/level/lib/lod/level-lod-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { usePartialChange } from "@/lib/react/use-partial-change";

interface ILevelLodActionProps extends BaseComponentProps {
  isOn: boolean;
  /** Whether the renderer's settings draw impostors at all, which this view can only narrow. */
  isAvailable: boolean;
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
  isAvailable,
  lod,
  onToggle,
  onChange,
}: ILevelLodActionProps): ReactElement {
  const onSet = usePartialChange(lod, onChange);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Impostors"}
      description={describeLevelFeatureToggle({
        isAvailable,
        isOn,
        isPlural: true,
        label: "Impostors",
        off: "Impostors off, every tree drawn",
        on: `Impostors past ${formatLevelLodDistance(lod.distance)} the game's distance`,
      })}
      icon={<ForestIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Draw distant trees as impostors"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Distance"}
        value={lod.distance}
        {...LEVEL_LOD_LIMITS.distance}
        format={formatLevelLodDistance}
        onChange={(distance: number) => onSet({ distance })}
      />

      <Button size={"small"} onClick={() => onChange(DEFAULT_LEVEL_LOD_OPTIONS)}>
        Back to the game&apos;s distance
      </Button>
    </EditorPopoverToggle>
  );
}
