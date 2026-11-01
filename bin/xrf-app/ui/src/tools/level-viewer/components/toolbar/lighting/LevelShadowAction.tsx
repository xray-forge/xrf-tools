import { ERenderContactShadowMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { TonalityIcon } from "@xrf/ui/icons/tonality";
import { ReactElement, useCallback } from "react";

import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_CONTACT_SHADOW_SETTINGS } from "@/core/render/lib/features/lighting/render-contact-shadow-choices";
import {
  RENDER_SHADOW_CASCADE_OPTIONS,
  RENDER_SHADOW_RESOLUTION_OPTIONS,
  RENDER_SHADOW_SETTINGS,
} from "@/core/render/lib/features/lighting/render-shadow-choices";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/choice/render-feature-defaults";
import {
  TRenderContactShadowSettings,
  TRenderShadowSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * Whether the sun casts shadows in this view, in how many cascades, how fine and how soft, how often and how far out
 * the cascades redraw swaying trees, and whether contact shadows found in the frame's depth darken what the cascades
 * are too coarse to.
 */
export function LevelShadowAction({
  "data-testid": dataTestId = "level-shadow-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { isOn, toggle } = useLevelViewToggle("isShadowed");
  const { state, set, setNested, reset } = useLevelFeature("shadows");
  const { drawn } = useLevelRenderFeatures();
  const shadows: TRenderShadowSettings = state.value;
  const widest: number = shadows.cascades.at(-1) ?? 0;
  const isContactShadowed: boolean = shadows.contact.mode === ERenderContactShadowMode.ENHANCED;

  const onSetContact = useCallback(
    (part: Partial<TRenderContactShadowSettings>): void => setNested("contact", part),
    [setNested]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun shadows"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        isPlural: true,
        label: "Sun shadows",
        off: "Sun shadows off: the sun lights every surface facing it",
        on:
          `Sun shadows in ${shadows.cascades.length} cascades, the widest ${widest} m across, at ${shadows.resolution}` +
          (isContactShadowed ? ", with contact shadows" : ""),
      })}
      icon={<TonalityIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Sun shadows"}
      onToggle={toggle}
    >
      <RenderValueChoice
        label={RENDER_SHADOW_SETTINGS.cascades.label}
        options={RENDER_SHADOW_CASCADE_OPTIONS}
        value={String(shadows.cascades.length)}
        onChange={(count: string) => set({ cascades: RENDER_SHADOW_CASCADE_WIDTHS.slice(0, Number(count)) })}
      />

      <RenderValueChoice
        label={RENDER_SHADOW_SETTINGS.resolution.label}
        options={RENDER_SHADOW_RESOLUTION_OPTIONS}
        value={String(shadows.resolution)}
        onChange={(resolution: string) => set({ resolution: Number(resolution) })}
      />

      <RenderSettingEntries
        settings={RENDER_SHADOW_SETTINGS}
        names={["filter", "bias", "blend", "swayRate", "swayCascades"]}
        group={shadows}
        features={drawn}
        onSet={set}
      />

      <RenderSettingEntries
        settings={RENDER_CONTACT_SHADOW_SETTINGS}
        names={["mode", "length", "intensity", "thickness", "steps", "isFullResolution", "lights"]}
        group={shadows.contact}
        features={drawn}
        onSet={onSetContact}
      />

      <LevelSettingsResetButton onReset={reset} />
    </EditorPopoverToggle>
  );
}
