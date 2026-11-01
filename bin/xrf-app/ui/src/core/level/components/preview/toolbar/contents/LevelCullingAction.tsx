import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { LayersIcon } from "@xrf/ui/icons/layers";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import {
  DEFAULT_LEVEL_LOD_OPTIONS,
  formatLevelLodDistance,
  ILevelLodOptions,
  LEVEL_LOD_LIMITS,
} from "@/core/level/lib/lod/level-lod-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { describeRenderSetting } from "@/core/render/lib/features/render-setting";
import { RENDER_CULLING_SETTINGS, RENDER_LOD_SETTINGS } from "@/core/render/lib/features/world/render-culling-choices";

/** What a section says of a culling the settings keep off, which this view cannot turn on. */
const OFF_IN_SETTINGS: string = "Off in Settings, under Rendering";

/**
 * What is spared drawing: static draws the depth hides, opaque pixels shaded more than once, and distant trees drawn as
 * their impostors.
 */
export function LevelCullingAction({
  "data-testid": dataTestId = "level-culling-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings } = useLevelRenderFeatures();

  const options: ILevelViewOptions = viewService.options;
  const lod: ILevelLodOptions = viewService.lod;
  // Whether the settings cull by occlusion, draw the depth pass and draw impostors at all, which this view can only
  // narrow.
  const isOcclusionAvailable: boolean = settings.isOcclusionCulled;
  const isPrepassAvailable: boolean = settings.isDepthPrepassed;
  const isImpostorsAvailable: boolean = settings.lod.isImpostors;
  const isOccluding: boolean = options.isOcclusionCulled && isOcclusionAvailable;
  const isPrepassed: boolean = options.isDepthPrepassed && isPrepassAvailable;
  const isImpostors: boolean = options.isImpostors && isImpostorsAvailable;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Culling"}
      description={[
        isOccluding ? "Occlusion culled" : "Occlusion culling off",
        isPrepassed ? "opaque depth first" : null,
        isImpostors ? `impostors past ${formatLevelLodDistance(lod.distance)} the game's distance` : "every tree drawn",
      ]
        .filter(Boolean)
        .join(", ")}
      icon={<LayersIcon />}
      isActive={isOccluding || isPrepassed || isImpostors}
    >
      <EditorPopoverGroupSection
        label={RENDER_CULLING_SETTINGS.isOcclusionCulled.label}
        description={
          isOcclusionAvailable
            ? describeRenderSetting(RENDER_CULLING_SETTINGS.isOcclusionCulled, settings, settings)
            : OFF_IN_SETTINGS
        }
        isOn={isOccluding}
        isDisabled={!isOcclusionAvailable}
        onToggle={() => viewService.toggleOption("isOcclusionCulled")}
      />

      <EditorPopoverGroupSection
        label={RENDER_CULLING_SETTINGS.isDepthPrepassed.label}
        description={
          isPrepassAvailable
            ? describeRenderSetting(RENDER_CULLING_SETTINGS.isDepthPrepassed, settings, settings)
            : OFF_IN_SETTINGS
        }
        isOn={isPrepassed}
        isDisabled={!isPrepassAvailable}
        onToggle={() => viewService.toggleOption("isDepthPrepassed")}
      />

      <EditorPopoverGroupSection
        label={RENDER_LOD_SETTINGS.isImpostors.label}
        description={
          isImpostorsAvailable
            ? describeRenderSetting(RENDER_LOD_SETTINGS.isImpostors, settings.lod, settings)
            : OFF_IN_SETTINGS
        }
        isOn={isImpostors}
        isDisabled={!isImpostorsAvailable}
        onToggle={() => viewService.toggleOption("isImpostors")}
      >
        <RenderValueSlider
          label={"Distance"}
          value={lod.distance}
          {...LEVEL_LOD_LIMITS.distance}
          format={formatLevelLodDistance}
          onChange={(distance: number) => viewService.setLod({ ...lod, distance })}
        />

        <Button size={"small"} onClick={() => viewService.setLod(DEFAULT_LEVEL_LOD_OPTIONS)}>
          Reset to the game&apos;s
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
