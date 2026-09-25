import { default as TonalityIcon } from "@mui/icons-material/Tonality";
import { Button } from "@mui/material";
import { IRendererShadowSettings } from "@xrf/renderer";
import { ReactElement, useCallback } from "react";

import { ILevelFeatureOptions, TLevelShadowOptions } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatCascadeBlend,
  RENDER_SHADOW_CASCADE_WIDTHS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTIONS,
} from "@/core/render/lib/features/render-feature-choices";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

const CASCADE_OPTIONS = RENDER_SHADOW_CASCADE_WIDTHS.map((_, index: number) => ({
  label: String(index + 1),
  value: String(index + 1),
}));

const RESOLUTION_OPTIONS = RENDER_SHADOW_RESOLUTIONS.map((value: number) => ({
  label: String(value),
  value: String(value),
}));

interface ILevelShadowActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The shadows the view is drawn with: the settings', with the view's own values over them. */
  shadows: IRendererShadowSettings;
  /** Whether the renderer's settings draw shadows at all, which this view can only narrow. */
  isAvailable?: boolean;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * Whether the sun casts shadows in this view, and in how many cascades, how fine and how soft.
 */
export function LevelShadowAction({
  "data-testid": dataTestId = "level-shadow-action",
  id,
  className,
  isOn,
  shadows,
  isAvailable = true,
  features,
  onToggle,
  onChange,
}: ILevelShadowActionProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelShadowOptions>) => onChange({ ...features, shadows: { ...features.shadows, ...part } }),
    [features, onChange]
  );

  const widest: number = shadows.cascades.at(-1) ?? 0;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Shadows"}
      description={
        !isAvailable
          ? "Shadows are off in Settings, under Rendering"
          : isOn
            ? `Shadows in ${shadows.cascades.length} cascades, the widest ${widest} m across, at ${shadows.resolution}`
            : "Shadows off, the sun lights every surface facing it"
      }
      icon={<TonalityIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Cast the sun's shadows"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Cascades"}
        options={CASCADE_OPTIONS}
        value={String(shadows.cascades.length)}
        onChange={(count: string) => set({ cascades: RENDER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
      />

      <RenderValueChoice
        label={"Map resolution"}
        options={RESOLUTION_OPTIONS}
        value={String(shadows.resolution)}
        onChange={(resolution: string) => set({ resolution: Number(resolution) })}
      />

      <RenderValueSlider
        label={"Filter"}
        value={shadows.filter}
        {...RENDER_SHADOW_LIMITS.filter}
        format={(value: number) => (value ? `${value} texel${value > 1 ? "s" : ""}` : "Hard")}
        onChange={(filter: number) => set({ filter })}
      />

      <RenderValueSlider
        label={"Normal offset"}
        value={shadows.bias}
        {...RENDER_SHADOW_LIMITS.bias}
        format={(value: number) => formatNumber(value, 2)}
        onChange={(bias: number) => set({ bias })}
      />

      <RenderValueSlider
        label={"Cascade blend"}
        value={shadows.blend}
        {...RENDER_SHADOW_LIMITS.blend}
        format={formatCascadeBlend}
        onChange={(blend: number) => set({ blend })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, shadows: {} })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
