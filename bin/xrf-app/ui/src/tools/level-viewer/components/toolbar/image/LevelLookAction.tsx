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
import { Optional } from "@xrf/types";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverCheckboxRow, EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ChoiceListFormRow } from "@xrf/ui/form/choice-list-row";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { ExposureIcon } from "@xrf/ui/icons/exposure";
import { ReactElement, useCallback } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingEntry } from "@/core/render/components/settings/RenderSettingEntry";
import {
  explainRenderBloomMode,
  RENDER_BLOOM_MODE_OPTIONS,
  RENDER_ENHANCED_BLOOM_SETTINGS,
} from "@/core/render/lib/features/tone/render-bloom-choices";
import { explainRenderDisplayTransform } from "@/core/render/lib/features/tone/render-display-transform-choices";
import { RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/features/tone/render-exposure-choices";
import { RENDER_GRADING_SETTINGS } from "@/core/render/lib/features/tone/render-grading-choices";
import {
  explainRenderLightLevel,
  getRenderLightLevelOptions,
} from "@/core/render/lib/features/tone/render-light-level-choices";
import {
  explainRenderLightModel,
  RENDER_LIGHT_SETTINGS,
} from "@/core/render/lib/features/tone/render-light-model-choices";
import {
  TRenderEnhancedBloomSettings,
  TRenderExposureSettings,
  TRenderGradingSettings,
  TRenderLightSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { LevelLookSliders } from "@/tools/level-viewer/components/toolbar/image/LevelLookSliders";
import {
  LEVEL_LOOK_BLOOM_FIELDS,
  LEVEL_LOOK_IMAGE_FIELDS,
  LEVEL_LOOK_LIGHT_SCALE_FIELDS,
  LEVEL_LOOK_TONEMAP_FIELDS,
} from "@/tools/level-viewer/lib/look/level-look-fields";

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
 * or values edited by hand, a section each of the light, the exposure, the display, the bloom (the engine's values
 * first, the enhanced bloom's after them) and the image; and the sky's debanding last.
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
  const { set: setBloom } = useLevelFeatureOverride("enhancedBloom");
  const onBloomReset: Optional<() => void> = useLevelFeatureReset("enhancedBloom");
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
  const isDevLevelAvailable: boolean = renderSettingsService.isDevLightLevelAvailable;
  const lightLevel: RenderLightLevel = drawn.lightLevel;
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

  const onSetLight = useCallback(
    (part: Partial<TRenderLightSettings>): void => viewService.setFeatures({ ...features, ...part }),
    [viewService, features]
  );

  // The look's own exposure, which its source presets, edited as the setting it is declares.
  const onSetExposure = useCallback(
    (part: Partial<TRenderExposureSettings>): void =>
      lookService.edit({ ...look, exposure: { ...look.exposure, ...part } }),
    [lookService, look]
  );

  const onEngineBloom = useCallback(
    (isEnabled: boolean): void => lookService.edit({ ...look, bloom: { ...look.bloom, isEnabled } }),
    [lookService, look]
  );

  const onBloomMode = useCallback((mode: RenderBloomMode): void => setBloom({ mode }), [setBloom]);

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
      <EditorPopoverContent label={"Look"} isWide>
        <ChoiceListFormRow
          data-testid={"level-look-source"}
          label={"Source"}
          description={"Picks where the exposure, light scales, bloom and image corrections come from"}
          options={options}
          value={picked}
          filterFrom={options.length + 1}
          onChange={onPick}
        />

        <EditorPopoverGroupSection label={"Light"} note={explainRenderLightModel(lightModel)}>
          <RenderSettingEntry
            settings={RENDER_LIGHT_SETTINGS}
            name={"lightModel"}
            group={drawn}
            features={drawn}
            onSet={onSetLight}
          />

          <RenderSettingEntry
            settings={RENDER_LIGHT_SETTINGS}
            name={"lightLevel"}
            group={drawn}
            features={drawn}
            options={getRenderLightLevelOptions(isDevLevelAvailable)}
            onSet={onSetLight}
          />

          {isPhysical ? <EditorPopoverNote>{explainRenderLightLevel(lightLevel)}</EditorPopoverNote> : null}

          <RenderSettingEntry
            settings={RENDER_LIGHT_SETTINGS}
            name={"sunTint"}
            group={drawn}
            features={drawn}
            onSet={onSetLight}
          />

          <LevelLookSliders
            fields={LEVEL_LOOK_LIGHT_SCALE_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection label={"Exposure"}>
          <RenderSettingEntries
            settings={RENDER_EXPOSURE_SETTINGS}
            names={["isEnabled", "compensation"]}
            group={look.exposure}
            features={drawn}
            onSet={onSetExposure}
          />

          <LevelLookSliders
            fields={LEVEL_LOOK_TONEMAP_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection
          label={"Display"}
          note={isPhysical ? explainRenderDisplayTransform(displayTransform) : "Under the physical light only."}
        >
          <RenderSettingEntry
            settings={RENDER_LIGHT_SETTINGS}
            name={"displayTransform"}
            group={drawn}
            features={drawn}
            onSet={onSetLight}
          />

          <RenderSettingEntries
            settings={RENDER_GRADING_SETTINGS}
            names={["contrast", "saturation"]}
            group={grading}
            features={drawn}
            onSet={setGrading}
          />
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection
          label={"Bloom"}
          note={explainRenderBloomMode(bloom.mode)}
          resetLabel={LEVEL_SETTINGS_RESET_LABEL}
          onReset={onBloomReset}
        >
          <RenderValueChoice
            label={"Mode"}
            options={RENDER_BLOOM_MODE_OPTIONS}
            value={bloom.mode}
            onChange={onBloomMode}
          />

          <EditorPopoverCheckboxRow
            label={"Engine bloom"}
            description={"The engine's bloom: blurs the frame's bright parts over it. Game setting: phase_bloom."}
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

          <RenderSettingEntries
            settings={RENDER_ENHANCED_BLOOM_SETTINGS}
            names={["threshold", "exposure", "blur", "vibrance", "sky"]}
            group={bloom}
            features={drawn}
            onSet={setBloom}
          />
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection label={"Image"} description={"The engine's corrections of the finished frame."}>
          <LevelLookSliders
            fields={LEVEL_LOOK_IMAGE_FIELDS}
            look={look}
            isPhysical={isPhysical}
            isPhysicalLevels={isPhysicalLevels}
            onEdit={onEdit}
          />
        </EditorPopoverGroupSection>

        <LevelEnhancedFeatureSection feature={"debanding"} />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
