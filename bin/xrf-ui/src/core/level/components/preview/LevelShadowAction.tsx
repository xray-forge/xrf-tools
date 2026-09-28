import { default as TonalityIcon } from "@mui/icons-material/Tonality";
import { Button } from "@mui/material";
import { IRendererShadowSettings, RENDERER_SHADOW_CASCADE_WIDTHS } from "@xrf/renderer";
import { ReactElement } from "react";

import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatCascadeBlend,
  formatShadowBias,
  formatShadowFilter,
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";

/**
 * Whether the sun casts shadows in this view, and in how many cascades, how fine and how soft.
 */
export function LevelShadowAction({
  "data-testid": dataTestId = "level-shadow-action",
  id,
  className,
  isOn,
  state,
  features,
  onToggle,
  onChange,
}: ILevelFeatureActionProps<"shadows">): ReactElement {
  const { set, reset } = useLevelFeatureOverride("shadows", features, onChange);
  const shadows: IRendererShadowSettings = state.value;
  const widest: number = shadows.cascades.at(-1) ?? 0;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Shadows"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        isPlural: true,
        label: "Shadows",
        off: "Shadows off, the sun lights every surface facing it",
        on: `Shadows in ${shadows.cascades.length} cascades, the widest ${widest} m across, at ${shadows.resolution}`,
      })}
      icon={<TonalityIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Cast the sun's shadows"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Cascades"}
        options={RENDER_SHADOW_CASCADE_OPTIONS}
        value={String(shadows.cascades.length)}
        onChange={(count: string) => set({ cascades: RENDERER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
      />

      <RenderValueChoice
        label={"Map resolution"}
        options={RENDER_SHADOW_RESOLUTION_OPTIONS}
        value={String(shadows.resolution)}
        onChange={(resolution: string) => set({ resolution: Number(resolution) })}
      />

      <RenderValueSlider
        label={"Filter"}
        value={shadows.filter}
        {...RENDER_SHADOW_LIMITS.filter}
        format={formatShadowFilter}
        onChange={(filter: number) => set({ filter })}
      />

      <RenderValueSlider
        label={"Normal offset"}
        value={shadows.bias}
        {...RENDER_SHADOW_LIMITS.bias}
        format={formatShadowBias}
        onChange={(bias: number) => set({ bias })}
      />

      <RenderValueSlider
        label={"Cascade blend"}
        value={shadows.blend}
        {...RENDER_SHADOW_LIMITS.blend}
        format={formatCascadeBlend}
        onChange={(blend: number) => set({ blend })}
      />

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
