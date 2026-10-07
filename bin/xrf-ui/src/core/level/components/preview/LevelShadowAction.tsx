import { default as TonalityIcon } from "@mui/icons-material/Tonality";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ERenderContactShadowMode, RenderContactShadowMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatCascadeBlend,
  formatContactShadowIntensity,
  formatContactShadowLength,
  formatContactShadowLights,
  formatContactShadowSteps,
  formatContactShadowThickness,
  formatShadowBias,
  formatShadowFilter,
  IRenderChoiceOption,
  RENDER_CONTACT_SHADOW_LIMITS,
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";

const CONTACT_MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderContactShadowMode>> = [
  { label: "Engine", value: ERenderContactShadowMode.ENGINE },
  { label: "Enhanced", value: ERenderContactShadowMode.ENHANCED },
];

/**
 * Whether the sun casts shadows in this view, in how many cascades, how fine and how soft, and whether contact
 * shadows found in the frame's depth darken what the cascades are too coarse to.
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
  const shadows: TRenderShadowSettings = state.value;
  const contact: TRenderShadowSettings["contact"] = shadows.contact;
  const widest: number = shadows.cascades.at(-1) ?? 0;
  const isContactShadowed: boolean = contact.mode === ERenderContactShadowMode.ENHANCED;

  function setContact(part: Partial<TRenderShadowSettings["contact"]>): void {
    set({ contact: { ...contact, ...part } });
  }

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
        on:
          `Shadows in ${shadows.cascades.length} cascades, the widest ${widest} m across, at ${shadows.resolution}` +
          (isContactShadowed ? ", with contact shadows" : ""),
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
        onChange={(count: string) => set({ cascades: RENDER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
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

      <RenderValueChoice
        label={"Contact shadows"}
        options={CONTACT_MODE_OPTIONS}
        value={contact.mode}
        onChange={(mode: RenderContactShadowMode) => setContact({ mode })}
      />

      {isContactShadowed ? (
        <>
          <RenderValueSlider
            label={"Contact length"}
            value={contact.length}
            {...RENDER_CONTACT_SHADOW_LIMITS.length}
            format={formatContactShadowLength}
            onChange={(length: number) => setContact({ length })}
          />

          <RenderValueSlider
            label={"Contact intensity"}
            value={contact.intensity}
            {...RENDER_CONTACT_SHADOW_LIMITS.intensity}
            format={formatContactShadowIntensity}
            onChange={(intensity: number) => setContact({ intensity })}
          />

          <RenderValueSlider
            label={"Contact thickness"}
            value={contact.thickness}
            {...RENDER_CONTACT_SHADOW_LIMITS.thickness}
            format={formatContactShadowThickness}
            onChange={(thickness: number) => setContact({ thickness })}
          />

          <RenderValueSlider
            label={"Contact steps"}
            value={contact.steps}
            {...RENDER_CONTACT_SHADOW_LIMITS.steps}
            format={formatContactShadowSteps}
            onChange={(steps: number) => setContact({ steps })}
          />

          <RenderValueSlider
            label={"Contact lights"}
            value={contact.lights}
            {...RENDER_CONTACT_SHADOW_LIMITS.lights}
            format={formatContactShadowLights}
            onChange={(lights: number) => setContact({ lights })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
