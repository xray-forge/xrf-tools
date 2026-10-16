import { default as TextureIcon } from "@mui/icons-material/Texture";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ERenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { LevelParallaxSettings } from "@/core/level/components/preview/LevelParallaxSettings";
import { useLevelRenderFeatures } from "@/core/level/components/preview/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ELevelShading, getLevelShading, LEVEL_SHADINGS } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Every shading as the list offers it. */
const SHADING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ELevelShading>> = LEVEL_SHADINGS.map(({ label, value }) => ({
  label,
  value,
}));

/**
 * How the level is shaded: the frame as dressed, as clay or by shader, or one of the targets it is built from; and the
 * surfaces as wireframe, with their bumps and parallax, under their wall marks, whichever of those is shown.
 */
export function LevelShadingAction({
  "data-testid": dataTestId = "level-shading-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings, features } = useLevelRenderFeatures();

  const options: ILevelViewOptions = viewService.options;
  const shading: ELevelShading = viewService.shading;
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
        description={"The frame as dressed, as clay, by shader, or one of the targets it is built from"}
        options={SHADING_OPTIONS}
        value={shading}
        filterFrom={SHADING_OPTIONS.length + 1}
        onChange={viewService.setShading}
      />

      <EditorPopoverGroupSection
        label={"Wireframe"}
        isOn={isWireframe}
        onToggle={() => viewService.toggleOption("isWireframe")}
      />

      <EditorPopoverGroupSection
        label={"Bumps"}
        description={isBumped ? "Shaded with the bump pairs their textures declare" : "Shaded flat, as though unbound"}
        isOn={isBumped}
        onToggle={() => viewService.toggleOption("isBumped")}
      />

      <EditorPopoverGroupSection
        label={"Parallax"}
        description={
          isParallaxAvailable
            ? isParallaxed
              ? "Surfaces displaced by the height their textures carry"
              : "Every surface flat, its height unread"
            : "No parallax is drawn: neither the steep nor the enhanced"
        }
        isOn={isParallaxed}
        isDisabled={!isParallaxAvailable}
        onToggle={() => viewService.toggleOption("isParallaxed")}
      >
        <LevelParallaxSettings />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Wall marks"}
        description={isWallmarked ? "The decals compiled into the level, over what they mark" : "The surfaces bare"}
        isOn={isWallmarked}
        onToggle={() => viewService.toggleOption("isWallmarked")}
      />
    </EditorPopoverGroup>
  );
}
