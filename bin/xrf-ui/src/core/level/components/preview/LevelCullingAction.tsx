import { default as LayersIcon } from "@mui/icons-material/Layers";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import {
  DEFAULT_LEVEL_LOD_OPTIONS,
  formatLevelLodDistance,
  ILevelLodOptions,
  LEVEL_LOD_LIMITS,
} from "@/core/level/lib/lod/level-lod-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelCullingActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** Whether the settings cull by occlusion at all, which this view can only narrow. */
  isOcclusionAvailable: boolean;
  /** Whether the settings draw impostors at all. */
  isImpostorsAvailable: boolean;
  lod: ILevelLodOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
}

/**
 * What is spared drawing: static draws the depth hides, and distant trees drawn as their impostors.
 */
export function LevelCullingAction({
  "data-testid": dataTestId = "level-culling-action",
  id,
  className,
  options,
  isOcclusionAvailable,
  isImpostorsAvailable,
  lod,
  onToggle,
  onChangeLod,
}: ILevelCullingActionProps): ReactElement {
  const isOccluding: boolean = options.isOcclusionCulled && isOcclusionAvailable;
  const isImpostors: boolean = options.isImpostors && isImpostorsAvailable;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Culling"}
      description={[
        isOccluding ? "Occlusion culled" : "Occlusion culling off",
        isImpostors ? `impostors past ${formatLevelLodDistance(lod.distance)} the game's distance` : "every tree drawn",
      ].join(", ")}
      icon={<LayersIcon />}
      isActive={isOccluding || isImpostors}
    >
      <EditorPopoverGroupSection
        label={"Occlusion culling"}
        description={
          isOcclusionAvailable ? "Culls what the depth hides before it draws" : "Off in Settings, under Rendering"
        }
        isOn={isOccluding}
        isDisabled={!isOcclusionAvailable}
        onToggle={() => onToggle("isOcclusionCulled")}
      />

      <EditorPopoverGroupSection
        label={"Impostors"}
        description={isImpostorsAvailable ? "Draws distant trees as impostors" : "Off in Settings, under Rendering"}
        isOn={isImpostors}
        isDisabled={!isImpostorsAvailable}
        onToggle={() => onToggle("isImpostors")}
      >
        <RenderValueSlider
          label={"Distance"}
          value={lod.distance}
          {...LEVEL_LOD_LIMITS.distance}
          format={formatLevelLodDistance}
          onChange={(distance: number) => onChangeLod({ ...lod, distance })}
        />

        <Button size={"small"} onClick={() => onChangeLod(DEFAULT_LEVEL_LOD_OPTIONS)}>
          Back to the game&apos;s distance
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
