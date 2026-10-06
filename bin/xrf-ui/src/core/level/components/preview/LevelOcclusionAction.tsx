import { default as GradientIcon } from "@mui/icons-material/Gradient";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import {
  ERenderAmbientOcclusionMethod,
  ERenderIndirectLightMode,
  RenderAmbientOcclusionMethod,
  RenderAmbientOcclusionQuality,
} from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { LevelIndirectLightSection } from "@/core/level/components/preview/LevelIndirectLightSection";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { DEFAULT_LEVEL_HEMI_STRENGTH, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderAmbientOcclusionMethod,
  describeRenderAmbientOcclusionQuality,
  explainRenderAmbientOcclusionMethod,
  formatOcclusionAccumulation,
  formatOcclusionBounce,
  formatOcclusionRadius,
  formatOcclusionStrength,
  formatOcclusionThickness,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
  RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS,
  RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS,
} from "@/core/render/lib/features";
import {
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
  TRenderIndirectLightSettings,
} from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { formatPercent } from "@/lib/format/number";

interface ILevelOcclusionActionProps extends Omit<ILevelFeatureActionProps<"ambientOcclusion">, "isOn" | "onToggle"> {
  options: ILevelViewOptions;
  /** How much the baked hemisphere darkens the ambient. */
  hemiStrength: number;
  /** The indirect light as the view draws it, which the occlusion's search gathers. */
  indirectLight: TRenderIndirectLightSettings;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onChangeHemiStrength: (hemiStrength: number) => void;
}

/**
 * What darkens creases and corners and what lights them back: the screen's ambient occlusion, GTAO (XeGTAO) or VBAO,
 * whose visibility bitmask lets light through behind thin things, the indirect light the same search gathers, and the
 * hemisphere occlusion xrLC baked.
 */
export function LevelOcclusionAction({
  "data-testid": dataTestId = "level-occlusion-action",
  id,
  className,
  options,
  state,
  features,
  hemiStrength,
  indirectLight,
  onToggle,
  onChange,
  onChangeHemiStrength,
}: ILevelOcclusionActionProps): ReactElement {
  const { set, reset } = useLevelFeatureOverride("ambientOcclusion", features, onChange);
  const occlusion: TRenderAmbientOcclusionSettings = state.value;
  const vbao: TRenderAmbientOcclusionVbaoSettings = occlusion.vbao;
  const isScreen: boolean = options.isOccluded && state.isAvailable;
  const isVbao: boolean = occlusion.method === ERenderAmbientOcclusionMethod.VBAO;
  const isIndirect: boolean = indirectLight.mode === ERenderIndirectLightMode.ENHANCED;

  const setVbao = useCallback(
    (part: Partial<TRenderAmbientOcclusionVbaoSettings>): void => {
      set({ vbao: { ...vbao, ...part } });
    },
    [vbao, set]
  );

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Occlusion"}
      description={[
        isScreen
          ? `${describeRenderAmbientOcclusionMethod(occlusion.method)} occlusion over ` +
            `${formatOcclusionRadius(occlusion.radius)}, ` +
            `${describeRenderAmbientOcclusionQuality(occlusion.quality).toLowerCase()} quality`
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
        onToggle={() => onToggle("isOccluded")}
      >
        <RenderValueChoice
          label={"Method"}
          options={RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS}
          value={occlusion.method}
          onChange={(method: RenderAmbientOcclusionMethod) => set({ method })}
        />

        <p className={"text-xs text-text-secondary"}>{explainRenderAmbientOcclusionMethod(occlusion.method)}</p>

        <RenderValueChoice
          label={"Quality"}
          options={RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS}
          value={occlusion.quality}
          onChange={(quality: RenderAmbientOcclusionQuality) => set({ quality })}
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

        {isVbao ? (
          <>
            <RenderValueSlider
              label={"Thickness"}
              value={vbao.thickness}
              {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.thickness}
              format={formatOcclusionThickness}
              onChange={(thickness: number) => setVbao({ thickness })}
            />

            <RenderValueSlider
              label={"Bounce"}
              value={vbao.bounce}
              {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.bounce}
              format={formatOcclusionBounce}
              onChange={(bounce: number) => setVbao({ bounce })}
            />

            <RenderValueSlider
              label={"Accumulation"}
              value={vbao.accumulation}
              {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.accumulation}
              format={formatOcclusionAccumulation}
              onChange={(accumulation: number) => setVbao({ accumulation })}
            />
          </>
        ) : null}

        <Button size={"small"} onClick={reset}>
          Back to the settings
        </Button>
      </EditorPopoverGroupSection>

      <LevelIndirectLightSection value={indirectLight} features={features} onChange={onChange} />

      <EditorPopoverGroupSection label={"Baked light"} isOn={options.isBaked} onToggle={() => onToggle("isBaked")}>
        <RenderValueSlider
          label={"Occlusion"}
          value={hemiStrength}
          min={0}
          max={1}
          step={0.05}
          format={formatPercent}
          onChange={onChangeHemiStrength}
        />

        <Button size={"small"} onClick={() => onChangeHemiStrength(DEFAULT_LEVEL_HEMI_STRENGTH)}>
          Back to the whole occlusion
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
