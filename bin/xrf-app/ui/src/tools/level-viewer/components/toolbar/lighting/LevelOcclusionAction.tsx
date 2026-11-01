import { useInjection } from "@wirestate/react";
import { formatPercent } from "@xrf/format";
import {
  ERenderAmbientOcclusionMethod,
  ERenderIndirectLightMode,
  RenderAmbientOcclusionMethod,
} from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GradientIcon } from "@xrf/ui/icons/gradient";
import { Button } from "@xrf/ui/material/button";
import { ReactElement, useCallback } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_HEMI_STRENGTH, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingEntry } from "@/core/render/components/settings/RenderSettingEntry";
import {
  describeRenderAmbientOcclusionMethod,
  describeRenderAmbientOcclusionSearch,
  explainRenderAmbientOcclusionMethod,
  formatOcclusionRadius,
  RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_METHOD_UNTRACED_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_SETTINGS,
  RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS,
} from "@/core/render/lib/features/tracing/render-ambient-occlusion-choices";
import {
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
  TRenderIndirectLightSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * What darkens creases and corners and what lights them back: the ambient occlusion, GTAO (XeGTAO), VBAO, whose
 * visibility bitmask lets light through behind thin things, or RTAO, traced against the level's geometry, which needs a
 * GPU that traces rays; the indirect light the bitmask search gathers; and the hemisphere occlusion xrLC baked.
 */
export function LevelOcclusionAction({
  "data-testid": dataTestId = "level-occlusion-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { state, set, setNested, reset } = useLevelFeature("ambientOcclusion");

  const options: ILevelViewOptions = viewService.options;
  const hemiStrength: number = viewService.hemiStrength;
  // The indirect light as the view draws it, which the occlusion's search gathers.
  const indirectLight: TRenderIndirectLightSettings = toLevelModeFeature("indirectLight", settings, features);
  const isTraceable: boolean = renderService.isTraceable;
  const occlusion: TRenderAmbientOcclusionSettings = state.value;
  const vbao: TRenderAmbientOcclusionVbaoSettings = occlusion.vbao;
  const isScreen: boolean = options.isOccluded && state.isAvailable;
  const isGtao: boolean = occlusion.method === ERenderAmbientOcclusionMethod.GTAO;
  const isRtao: boolean = occlusion.method === ERenderAmbientOcclusionMethod.RTAO;
  // VBAO and RTAO both gather over frames and bounce light back from their creases.
  const isGathered: boolean = occlusion.method !== ERenderAmbientOcclusionMethod.GTAO;
  const isUntraced: boolean = isRtao && !isTraceable;
  // Whether VBAO's bitmask searches: chosen, or in place of RTAO on a GPU that traces no rays.
  const isVbaoSearched: boolean = occlusion.method === ERenderAmbientOcclusionMethod.VBAO || isUntraced;
  const isIndirect: boolean = indirectLight.mode === ERenderIndirectLightMode.ENHANCED;

  const onSetVbao = useCallback(
    (part: Partial<TRenderAmbientOcclusionVbaoSettings>): void => setNested("vbao", part),
    [setNested]
  );
  const onSetGtao = useCallback(
    (part: Partial<TRenderAmbientOcclusionGtaoSettings>): void => setNested("gtao", part),
    [setNested]
  );
  const onSetRtao = useCallback(
    (part: Partial<TRenderAmbientOcclusionRtaoSettings>): void => setNested("rtao", part),
    [setNested]
  );

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Ambient light"}
      description={[
        isScreen
          ? `${describeRenderAmbientOcclusionMethod(occlusion.method)} occlusion over ` +
            `${formatOcclusionRadius(occlusion.radius)}, ` +
            describeRenderAmbientOcclusionSearch(occlusion)
          : "Ambient occlusion off",
        isIndirect ? "indirect light" : null,
        options.isBaked ? `baked at ${formatPercent(hemiStrength)}` : "baked off",
      ]
        .filter((it): it is string => it !== null)
        .join(", ")}
      icon={<GradientIcon />}
      isActive={isScreen || isIndirect || options.isBaked}
    >
      <EditorPopoverGroupSection
        label={"Ambient occlusion"}
        description={state.isAvailable ? undefined : "Off in Settings, under Rendering"}
        isOn={isScreen}
        isDisabled={!state.isAvailable}
        onToggle={() => viewService.toggleOption("isOccluded")}
      >
        <RenderValueChoice
          label={RENDER_AMBIENT_OCCLUSION_SETTINGS.method.label}
          options={
            isTraceable ? RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS : RENDER_AMBIENT_OCCLUSION_METHOD_UNTRACED_OPTIONS
          }
          value={occlusion.method}
          onChange={(method: RenderAmbientOcclusionMethod) => set({ method })}
        />

        <p className={"text-xs text-text-secondary"}>
          {isUntraced
            ? "RTAO needs a GPU that traces rays: VBAO searches in its place."
            : explainRenderAmbientOcclusionMethod(occlusion.method)}
        </p>

        {isGtao ? (
          <RenderSettingEntries
            settings={RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS}
            names={["slices", "steps"]}
            group={occlusion.gtao}
            features={drawn}
            onSet={onSetGtao}
          />
        ) : (
          // VBAO's search, which RTAO traces in place of and falls back to on a GPU that traces no rays.
          <RenderSettingEntries
            settings={RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS}
            names={["slices", "steps"]}
            group={vbao}
            features={drawn}
            isDisabled={!isVbaoSearched}
            onSet={onSetVbao}
          />
        )}

        <RenderSettingEntry
          settings={RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS}
          name={"rays"}
          group={occlusion.rtao}
          features={drawn}
          isDisabled={!isRtao || isUntraced}
          onSet={onSetRtao}
        />

        <RenderSettingEntries
          settings={RENDER_AMBIENT_OCCLUSION_SETTINGS}
          names={["radius", "strength"]}
          group={occlusion}
          features={drawn}
          onSet={set}
        />

        <RenderSettingEntry
          settings={RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS}
          name={"thickness"}
          group={vbao}
          features={drawn}
          isDisabled={!isVbaoSearched}
          onSet={onSetVbao}
        />

        <RenderSettingEntries
          settings={RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS}
          names={["bounce", "accumulation"]}
          group={vbao}
          features={drawn}
          isDisabled={!isGathered}
          onSet={onSetVbao}
        />

        <LevelSettingsResetButton subject={"ambient occlusion"} onReset={reset} />
      </EditorPopoverGroupSection>

      <LevelEnhancedFeatureSection feature={"indirectLight"} />

      <EditorPopoverGroupSection
        label={"Baked light"}
        isOn={options.isBaked}
        onToggle={() => viewService.toggleOption("isBaked")}
      >
        <RenderValueSlider
          label={"Baked hemisphere"}
          value={hemiStrength}
          min={0}
          max={1}
          step={0.05}
          format={formatPercent}
          onChange={viewService.setHemiStrength}
        />

        <Button size={"small"} onClick={() => viewService.setHemiStrength(DEFAULT_LEVEL_HEMI_STRENGTH)}>
          Reset to the game&apos;s
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
