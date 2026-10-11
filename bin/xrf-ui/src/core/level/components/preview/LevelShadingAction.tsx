import { default as TextureIcon } from "@mui/icons-material/Texture";
import { ReactElement } from "react";

import { ERenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { LevelParallaxSettings } from "@/core/level/components/preview/LevelParallaxSettings";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelShading, getLevelShading, LEVEL_SHADINGS } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Every shading as the list offers it. */
const SHADING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ELevelShading>> = LEVEL_SHADINGS.map(({ label, value }) => ({
  label,
  value,
}));

interface ILevelShadingActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** The parallax this view draws with while its toggle is on: the settings' and what the view sets over them. */
  parallax: TRenderParallaxSettings;
  /** What the view sets over the settings, the parallax's among them. */
  features: ILevelFeatureOptions;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
  /** What the viewport shows of its surfaces. */
  shading: ELevelShading;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onChangeShading: (shading: ELevelShading) => void;
}

/**
 * How the level is shaded: the frame as dressed, as clay or by shader, or one of the targets it is built from; and the
 * surfaces as wireframe, with their bumps and parallax, under their wall marks, whichever of those is shown.
 */
export function LevelShadingAction({
  "data-testid": dataTestId = "level-shading-action",
  id,
  className,
  options,
  parallax,
  features,
  shading,
  onToggle,
  onChangeShading,
  onChangeFeatures,
}: ILevelShadingActionProps): ReactElement {
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
        onChange={onChangeShading}
      />

      <EditorPopoverGroupSection label={"Wireframe"} isOn={isWireframe} onToggle={() => onToggle("isWireframe")} />

      <EditorPopoverGroupSection
        label={"Bumps"}
        description={isBumped ? "Shaded with the bump pairs their textures declare" : "Shaded flat, as though unbound"}
        isOn={isBumped}
        onToggle={() => onToggle("isBumped")}
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
        onToggle={() => onToggle("isParallaxed")}
      >
        <LevelParallaxSettings value={parallax} features={features} onChange={onChangeFeatures} />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Wall marks"}
        description={isWallmarked ? "The decals compiled into the level, over what they mark" : "The surfaces bare"}
        isOn={isWallmarked}
        onToggle={() => onToggle("isWallmarked")}
      />
    </EditorPopoverGroup>
  );
}
