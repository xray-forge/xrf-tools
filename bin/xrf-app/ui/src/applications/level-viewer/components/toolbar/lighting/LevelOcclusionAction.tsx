import { useInjection } from "@wirestate/react";
import { formatCount, formatPercent } from "@xrf/format";
import {
  ERenderAmbientOcclusionMethod,
  ERenderIndirectLightMode,
  RenderAmbientOcclusionMethod,
} from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GradientIcon } from "@xrf/ui/icons";
import { Button } from "@xrf/ui/material";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/shell/editor/EditorPopoverGroup";
import { ReactElement, useCallback } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_HEMI_STRENGTH, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderAmbientOcclusionMethod,
  describeRenderAmbientOcclusionSearch,
  explainRenderAmbientOcclusionMethod,
  formatOcclusionAccumulation,
  formatOcclusionBounce,
  formatOcclusionRadius,
  formatOcclusionStrength,
  formatOcclusionThickness,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
  RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_METHOD_UNTRACED_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS,
} from "@/core/render/lib/features/render-ambient-occlusion-choices";
import {
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
  TRenderIndirectLightSettings,
} from "@/core/render/lib/settings/render-feature-settings";

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
  const { settings, features } = useLevelRenderFeatures();
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
          label={"Method"}
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
          <>
            <RenderValueSlider
              label={"Slices"}
              value={occlusion.gtao.slices}
              {...RENDER_AMBIENT_OCCLUSION_LIMITS.gtaoSlices}
              format={formatCount}
              onChange={(slices: number) => onSetGtao({ slices })}
            />

            <RenderValueSlider
              label={"Steps"}
              value={occlusion.gtao.steps}
              {...RENDER_AMBIENT_OCCLUSION_LIMITS.gtaoSteps}
              format={formatCount}
              onChange={(steps: number) => onSetGtao({ steps })}
            />
          </>
        ) : (
          // VBAO's search, which RTAO traces in place of and falls back to on a GPU that traces no rays.
          <>
            <RenderValueSlider
              label={"Slices"}
              value={vbao.slices}
              {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.slices}
              isDisabled={!isVbaoSearched}
              format={formatCount}
              onChange={(slices: number) => onSetVbao({ slices })}
            />

            <RenderValueSlider
              label={"Steps"}
              value={vbao.steps}
              {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.steps}
              isDisabled={!isVbaoSearched}
              format={formatCount}
              onChange={(steps: number) => onSetVbao({ steps })}
            />
          </>
        )}

        <RenderValueSlider
          label={"Rays"}
          value={occlusion.rtao.rays}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.rays}
          isDisabled={!isRtao || isUntraced}
          format={formatCount}
          onChange={(rays: number) => onSetRtao({ rays })}
        />

        <RenderValueSlider
          label={"Radius"}
          value={occlusion.radius}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.radius}
          format={formatOcclusionRadius}
          onChange={(radius: number) => set({ radius })}
        />

        <RenderValueSlider
          label={"Strength"}
          value={occlusion.strength}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
          format={formatOcclusionStrength}
          onChange={(strength: number) => set({ strength })}
        />

        <RenderValueSlider
          label={"Thickness"}
          value={vbao.thickness}
          {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.thickness}
          isDisabled={!isVbaoSearched}
          format={formatOcclusionThickness}
          onChange={(thickness: number) => onSetVbao({ thickness })}
        />

        <RenderValueSlider
          label={"Bounce"}
          value={vbao.bounce}
          {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.bounce}
          isDisabled={!isGathered}
          format={formatOcclusionBounce}
          onChange={(bounce: number) => onSetVbao({ bounce })}
        />

        <RenderValueSlider
          label={"Accumulation"}
          value={vbao.accumulation}
          {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.accumulation}
          isDisabled={!isGathered}
          format={formatOcclusionAccumulation}
          onChange={(accumulation: number) => onSetVbao({ accumulation })}
        />

        <Button size={"small"} aria-label={"Reset the ambient occlusion to the preset"} onClick={reset}>
          Reset to the preset
        </Button>
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
