import { useInjection } from "@wirestate/react";
import { ERenderParallaxMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ChoiceListFormRow } from "@xrf/ui/form/choice-list-row";
import { TextureIcon } from "@xrf/ui/icons/texture";
import { ReactElement } from "react";

import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelParallaxSettings } from "@/core/level/components/preview/toolbar/view/LevelParallaxSettings";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import {
  ELevelShading,
  getLevelShading,
  getLevelShadings,
  ILevelShadingChoice,
} from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";

/**
 * How the level is shaded: the frame as dressed, as clay or by shader, or one of the targets it is built from, listed
 * by what they show, FSR's only while FSR draws the frame; and the surfaces as wireframe, with their bumps and parallax,
 * under their wall marks, whichever of those is shown.
 */
export function LevelShadingAction({
  "data-testid": dataTestId = "level-shading-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features } = useLevelRenderFeatures();

  const options: ILevelViewOptions = viewService.options;
  const shading: ELevelShading = renderService.shading;
  const shadings: ReadonlyArray<ILevelShadingChoice> = getLevelShadings(renderService.antialiasing);
  const parallax: TRenderParallaxSettings = toLevelModeFeature("parallax", settings, features);
  const { isWireframe, isBumped, isWallmarked } = options;
  const isParallaxAvailable: boolean = parallax.isSteep || parallax.mode === ERenderParallaxMode.ENHANCED;
  const isParallaxed: boolean = options.isParallaxed && isParallaxAvailable;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Shading"}
      description={[
        getLevelShading(shading).label,
        isWireframe ? "wireframe" : null,
        isBumped ? null : "flat",
        isParallaxed || !isParallaxAvailable ? null : "without parallax",
        isWallmarked ? null : "without wall marks",
      ]
        .filter((it): it is string => it !== null)
        .join(", ")}
      icon={<TextureIcon />}
      isActive={
        shading !== ELevelShading.FINAL ||
        isWireframe ||
        !isBumped ||
        (isParallaxAvailable && !isParallaxed) ||
        !isWallmarked
      }
    >
      <ChoiceListFormRow
        data-testid={"level-shading"}
        label={"Show"}
        options={shadings}
        value={shading}
        filterFrom={Infinity}
        onChange={viewService.setShading}
      />

      <EditorPopoverGroupSection
        label={"Wireframe"}
        isOn={isWireframe}
        onToggle={() => viewService.toggleOption("isWireframe")}
      />

      <EditorPopoverGroupSection
        label={"Bumps"}
        description={isBumped ? "Shades each surface with its textures' bump maps" : "Shades every surface flat"}
        isOn={isBumped}
        onToggle={() => viewService.toggleOption("isBumped")}
      />

      <EditorPopoverGroupSection
        label={"Parallax"}
        description={
          isParallaxAvailable
            ? isParallaxed
              ? "Displaces each surface by its textures' height"
              : "Draws every surface flat"
            : "Settings draw neither the steep nor the enhanced parallax"
        }
        isOn={isParallaxed}
        isDisabled={!isParallaxAvailable}
        onToggle={() => viewService.toggleOption("isParallaxed")}
      >
        <LevelParallaxSettings />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Wall marks"}
        description={
          isWallmarked ? "Draws the decals compiled into the level" : "Draws the surfaces without their decals"
        }
        isOn={isWallmarked}
        onToggle={() => viewService.toggleOption("isWallmarked")}
      />
    </EditorPopoverGroup>
  );
}
