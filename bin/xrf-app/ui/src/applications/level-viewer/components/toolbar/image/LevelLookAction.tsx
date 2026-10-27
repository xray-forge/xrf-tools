import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { LevelConsoleDefaults } from "@xrf/ipc/types/xrf-app-levels";
import {
  ERenderBloomMode,
  ERenderLightLevel,
  ERenderLightModel,
  RenderBloomMode,
  RenderDisplayTransform,
  RenderLightLevel,
  RenderLightModel,
} from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ExposureIcon } from "@xrf/ui/icons";
import { Button } from "@xrf/ui/material";
import { ReactElement, useCallback } from "react";

import { LevelLookSection } from "@/applications/level-viewer/components/toolbar/image/LevelLookSection";
import { LevelLookSliders } from "@/applications/level-viewer/components/toolbar/image/LevelLookSliders";
import {
  ILevelEnhancedBloomField,
  LEVEL_ENHANCED_BLOOM_FIELDS,
  LEVEL_LOOK_BLOOM_FIELDS,
  LEVEL_LOOK_IMAGE_FIELDS,
  LEVEL_LOOK_LIGHT_SCALE_FIELDS,
  LEVEL_LOOK_TONEMAP_FIELDS,
} from "@/applications/level-viewer/lib/look/level-look-fields";
import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderBloomMode,
  formatBloomStrength,
  RENDER_BLOOM_MODE_OPTIONS,
  RENDER_ENHANCED_BLOOM_LIMITS,
} from "@/core/render/lib/features/render-bloom-choices";
import {
  explainRenderDisplayTransform,
  RENDER_DISPLAY_TRANSFORM_OPTIONS,
} from "@/core/render/lib/features/render-display-transform-choices";
import { formatStops, RENDER_EXPOSURE_LIMITS } from "@/core/render/lib/features/render-exposure-choices";
import { formatGrading, RENDER_GRADING_LIMITS } from "@/core/render/lib/features/render-grading-choices";
import {
  explainRenderLightLevel,
  formatSunTint,
  getRenderLightLevelOptions,
  RENDER_SUN_TINT_LIMITS,
  toDrawnRenderLightLevel,
} from "@/core/render/lib/features/render-light-level-choices";
import {
  explainRenderLightModel,
  RENDER_LIGHT_MODEL_OPTIONS,
} from "@/core/render/lib/features/render-light-model-choices";
import {
  TRenderEnhancedBloomSettings,
  TRenderGradingSettings,
} from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";

/** What the list's option for values edited by hand stands for. */
const EDITED: string = "edited";

/** The looks offered whatever was saved, by their sources. */
const BUILT_IN_SOURCES: ReadonlyArray<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>> = [
  ELevelLookSource.GAME,
  ELevelLookSource.XRF,
  ELevelLookSource.ANOMALY,
  ELevelLookSource.GAMMA,
  ELevelLookSource.OPENXRAY,
  ELevelLookSource.SETTINGS,
];

/** Each built-in look's name in the list. */
const BUILT_IN_LABELS: Readonly<Record<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>, string>> = {
  [ELevelLookSource.GAME]: "Game defaults",
  [ELevelLookSource.XRF]: "XRF",
  [ELevelLookSource.ANOMALY]: "Anomaly",
  [ELevelLookSource.GAMMA]: "G.A.M.M.A.",
  [ELevelLookSource.OPENXRAY]: "OpenXRay",
  [ELevelLookSource.SETTINGS]: "Settings",
};

/**
 * @param defaults - What the open level's game ships as its console defaults, as far as they are read.
 * @returns What the game's look stands on beside its label: reading, unreadable, none shipped, or nothing to add.
 */
function describeGameDefaults(defaults: AsyncState<LevelConsoleDefaults>): string {
  if (defaults.isLoading) {
    return " (reading)";
  }

  if (defaults.isFailed) {
    return " (unreadable, the engine's)";
  }

  return defaults.value && !defaults.value.isShipped ? " (none shipped, the engine's)" : "";
}

/**
 * How the level is exposed, lit, bloomed and corrected: the game's console defaults, the settings', a built-in engine's,
 * or values edited by hand; whether the engine's bloom draws or the enhanced one in its place; and the sky's debanding.
 */
export function LevelLookAction({
  "data-testid": dataTestId = "level-look-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const lookService: LevelLookService = useInjection(LevelLookService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const { features, drawn } = useLevelRenderFeatures();
  const { set: setBloom, reset: resetBloom } = useLevelFeatureOverride("enhancedBloom");
  const { set: setGrading } = useLevelFeatureOverride("grading");

  // Which bloom the view draws, and the enhanced bloom's strengths.
  const bloom: TRenderEnhancedBloomSettings = drawn.enhancedBloom;
  // How the physical light's frame is graded before its display transform.
  const grading: TRenderGradingSettings = drawn.grading;
  const isEnhancedBloom: boolean = bloom.mode === ERenderBloomMode.ENHANCED;
  // What the view lights in: its own choice, else the settings'.
  const lightModel: RenderLightModel = drawn.lightModel;
  const isPhysical: boolean = lightModel === ERenderLightModel.PHYSICAL;
  const displayTransform: RenderDisplayTransform = drawn.displayTransform;
  const isPhysicalLevelAvailable: boolean = renderSettingsService.isPhysicalLightLevelAvailable;
  const lightLevel: RenderLightLevel = toDrawnRenderLightLevel(drawn.lightLevel, isPhysicalLevelAvailable);
  const isPhysicalLevels: boolean = isPhysical && lightLevel === ERenderLightLevel.PHYSICAL;

  const { choice, look, defaults } = lookService;
  const picked: string = choice.source === ELevelLookSource.CUSTOM ? EDITED : choice.source;
  const options: Array<IChoiceFormRowOption<string>> = [
    ...BUILT_IN_SOURCES.map((source) => ({
      label:
        source === ELevelLookSource.GAME
          ? `${BUILT_IN_LABELS[source]}${describeGameDefaults(defaults)}`
          : BUILT_IN_LABELS[source],
      value: source as string,
    })),
    // The values edited by hand stay offered once there are some, to go back to after trying another look.
    ...(choice.custom ? [{ label: "Edited by hand", value: EDITED }] : []),
  ];

  const onPick = useCallback(
    (value: string): void => {
      const source = value === EDITED ? ELevelLookSource.CUSTOM : BUILT_IN_SOURCES.find((it) => it === value);

      if (source) {
        lookService.setSource(source);
      }
    },
    [lookService]
  );

  const onEdit = useCallback((edited: ILevelLook): void => lookService.edit(edited), [lookService]);

  const onLightModel = useCallback(
    (lightModel: RenderLightModel): void => viewService.setFeatures({ ...features, lightModel }),
    [viewService, features]
  );

  const onLightLevel = useCallback(
    (lightLevel: RenderLightLevel): void => viewService.setFeatures({ ...features, lightLevel }),
    [viewService, features]
  );

  const onSunTint = useCallback(
    (sunTint: number): void => viewService.setFeatures({ ...features, sunTint }),
    [viewService, features]
  );

  const onDisplayTransform = useCallback(
    (displayTransform: RenderDisplayTransform): void => viewService.setFeatures({ ...features, displayTransform }),
    [viewService, features]
  );

  const onAutoExposure = useCallback(
    (isEnabled: boolean): void => lookService.edit({ ...look, exposure: { ...look.exposure, isEnabled } }),
    [lookService, look]
  );

  const onCompensation = useCallback(
    (compensation: number): void => lookService.edit({ ...look, exposure: { ...look.exposure, compensation } }),
    [lookService, look]
  );

  const onEngineBloom = useCallback(
    (isEnabled: boolean): void => lookService.edit({ ...look, bloom: { ...look.bloom, isEnabled } }),
    [lookService, look]
  );

  const onContrast = useCallback((contrast: number): void => setGrading({ contrast }), [setGrading]);

  const onSaturation = useCallback((saturation: number): void => setGrading({ saturation }), [setGrading]);

  const onBloomMode = useCallback((mode: RenderBloomMode): void => setBloom({ mode }), [setBloom]);

  const onBloomStrength = useCallback(
    (key: ILevelEnhancedBloomField["key"], value: number): void => setBloom({ [key]: value }),
    [setBloom]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Look"}
      description={`Exposure, light and image as ${options.find((it) => it.value === picked)?.label ?? "set"}`}
      icon={<ExposureIcon />}
      isActive={picked !== ELevelLookSource.GAME || isEnhancedBloom || isPhysical}
    >
      <div className={"flex w-80 flex-col gap-3 px-4 py-2"}>
        <LevelLookSection title={"Look"}>
          <ChoiceListFormRow
            data-testid={"level-look-source"}
            label={"Source"}
            description={"Picks where the exposure, light scales, bloom and image corrections come from"}
            options={options}
            value={picked}
            filterFrom={options.length + 1}
            onChange={onPick}
          />
        </LevelLookSection>

        <LevelLookSection title={"Light"} note={explainRenderLightModel(lightModel)}>
          <RenderValueChoice
            label={"Model"}
            options={RENDER_LIGHT_MODEL_OPTIONS}
            value={lightModel}
            onChange={onLightModel}
          />

          <RenderValueChoice
            label={"Levels"}
            options={getRenderLightLevelOptions(isPhysicalLevelAvailable)}
            value={lightLevel}
            isDisabled={!isPhysical}
            onChange={onLightLevel}
          />

          {isPhysical ? <p className={"text-xs text-text-secondary"}>{explainRenderLightLevel(lightLevel)}</p> : null}

          <RenderValueSlider
            label={"Sun tint"}
            value={drawn.sunTint}
            {...RENDER_SUN_TINT_LIMITS}
            isDisabled={!isPhysicalLevels}
            format={formatSunTint}
            onChange={onSunTint}
          />

          <LevelLookSliders
            fields={LEVEL_LOOK_LIGHT_SCALE_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />
        </LevelLookSection>

        <LevelLookSection title={"Exposure"}>
          <CheckboxFormRow
            label={"Auto exposure"}
            description={
              "Adapts the exposure to how bright the frame is; off, a fixed scale. Game setting: r2_tonemap."
            }
            isChecked={look.exposure.isEnabled}
            onChange={onAutoExposure}
          />

          <RenderValueSlider
            label={"Compensation"}
            value={look.exposure.compensation}
            {...RENDER_EXPOSURE_LIMITS.compensation}
            isDisabled={!isPhysical}
            format={formatStops}
            onChange={onCompensation}
          />

          <LevelLookSliders
            fields={LEVEL_LOOK_TONEMAP_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />
        </LevelLookSection>

        <LevelLookSection
          title={"Display"}
          note={isPhysical ? explainRenderDisplayTransform(displayTransform) : "Under the physical light only."}
        >
          <RenderValueChoice
            label={"Transform"}
            options={RENDER_DISPLAY_TRANSFORM_OPTIONS}
            value={displayTransform}
            isDisabled={!isPhysical}
            onChange={onDisplayTransform}
          />

          <RenderValueSlider
            label={"Contrast"}
            value={grading.contrast}
            {...RENDER_GRADING_LIMITS.contrast}
            isDisabled={!isPhysical}
            format={formatGrading}
            onChange={onContrast}
          />

          <RenderValueSlider
            label={"Saturation"}
            value={grading.saturation}
            {...RENDER_GRADING_LIMITS.saturation}
            isDisabled={!isPhysical}
            format={formatGrading}
            onChange={onSaturation}
          />
        </LevelLookSection>

        <LevelLookSection title={"Bloom"} note={explainRenderBloomMode(bloom.mode)}>
          <RenderValueChoice
            label={"Mode"}
            options={RENDER_BLOOM_MODE_OPTIONS}
            value={bloom.mode}
            onChange={onBloomMode}
          />

          <div className={"flex flex-col gap-1"}>
            {LEVEL_ENHANCED_BLOOM_FIELDS.map((field: ILevelEnhancedBloomField) => (
              <RenderValueSlider
                key={field.key}
                label={field.label}
                value={bloom[field.key]}
                {...RENDER_ENHANCED_BLOOM_LIMITS[field.key]}
                isDisabled={!isEnhancedBloom}
                format={formatBloomStrength}
                onChange={(value: number) => onBloomStrength(field.key, value)}
              />
            ))}
          </div>

          <CheckboxFormRow
            label={"Bloom"}
            description={"Blurs the frame's bright parts over it. Game setting: phase_bloom."}
            isChecked={look.bloom.isEnabled}
            isDisabled={isEnhancedBloom}
            onChange={onEngineBloom}
          />

          <LevelLookSliders
            fields={LEVEL_LOOK_BLOOM_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            isDisabled={isEnhancedBloom}
            onEdit={onEdit}
          />

          <Button size={"small"} aria-label={"Reset the bloom to the preset"} onClick={resetBloom}>
            Reset to the preset
          </Button>
        </LevelLookSection>

        <LevelLookSection
          title={"Image"}
          note={"The engine's corrections of the finished frame, and the sky's debanding."}
        >
          <LevelLookSliders
            fields={LEVEL_LOOK_IMAGE_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />

          <LevelEnhancedFeatureSection feature={"debanding"} />
        </LevelLookSection>
      </div>
    </EditorPopoverAction>
  );
}
