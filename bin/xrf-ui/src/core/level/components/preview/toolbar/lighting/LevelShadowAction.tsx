import { default as TonalityIcon } from "@mui/icons-material/Tonality";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderContactShadowMode, RenderContactShadowMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatContactShadowIntensity,
  formatContactShadowLength,
  formatContactShadowLights,
  formatContactShadowSteps,
  formatContactShadowThickness,
  RENDER_CONTACT_SHADOW_LIMITS,
  RENDER_CONTACT_SHADOW_MODE_OPTIONS,
} from "@/core/render/lib/features/render-contact-shadow-choices";
import {
  formatCascadeBlend,
  formatShadowBias,
  formatShadowFilter,
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_LIMITS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-shadow-choices";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Whether the sun casts shadows in this view, in how many cascades, how fine and how soft, and whether contact
 * shadows found in the frame's depth darken what the cascades are too coarse to.
 */
export function LevelShadowAction({
  "data-testid": dataTestId = "level-shadow-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { isOn, toggle } = useLevelViewToggle("isShadowed");
  const { state, set, reset } = useLevelFeature("shadows");
  const shadows: TRenderShadowSettings = state.value;
  const contact: TRenderShadowSettings["contact"] = shadows.contact;
  const widest: number = shadows.cascades.at(-1) ?? 0;
  const isContactShadowed: boolean = contact.mode === ERenderContactShadowMode.ENHANCED;

  const onSetContact = useCallback(
    (part: Partial<TRenderShadowSettings["contact"]>): void => {
      set({ contact: { ...contact, ...part } });
    },
    [contact, set]
  );

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
      onToggle={toggle}
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
        options={RENDER_CONTACT_SHADOW_MODE_OPTIONS}
        value={contact.mode}
        onChange={(mode: RenderContactShadowMode) => onSetContact({ mode })}
      />

      {isContactShadowed ? (
        <>
          <RenderValueSlider
            label={"Contact length"}
            value={contact.length}
            {...RENDER_CONTACT_SHADOW_LIMITS.length}
            format={formatContactShadowLength}
            onChange={(length: number) => onSetContact({ length })}
          />

          <RenderValueSlider
            label={"Contact intensity"}
            value={contact.intensity}
            {...RENDER_CONTACT_SHADOW_LIMITS.intensity}
            format={formatContactShadowIntensity}
            onChange={(intensity: number) => onSetContact({ intensity })}
          />

          <RenderValueSlider
            label={"Contact thickness"}
            value={contact.thickness}
            {...RENDER_CONTACT_SHADOW_LIMITS.thickness}
            format={formatContactShadowThickness}
            onChange={(thickness: number) => onSetContact({ thickness })}
          />

          <RenderValueSlider
            label={"Contact steps"}
            value={contact.steps}
            {...RENDER_CONTACT_SHADOW_LIMITS.steps}
            format={formatContactShadowSteps}
            onChange={(steps: number) => onSetContact({ steps })}
          />

          <CheckboxFormRow
            label={"Full resolution contact"}
            description={
              "Marched at every pixel rather than one in four: thin foliage stops flickering, at four times the cost."
            }
            isChecked={contact.isFullResolution}
            onChange={(isFullResolution: boolean) => onSetContact({ isFullResolution })}
          />

          <RenderValueSlider
            label={"Contact lights"}
            value={contact.lights}
            {...RENDER_CONTACT_SHADOW_LIMITS.lights}
            format={formatContactShadowLights}
            onChange={(lights: number) => onSetContact({ lights })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverToggle>
  );
}
