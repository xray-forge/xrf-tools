import { default as ExposureIcon } from "@mui/icons-material/Exposure";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { formatNumber } from "@xrf/format";
import { ReactElement, useCallback } from "react";

import { LevelDebandingSettings } from "@/applications/level-viewer/components/toolbar/image/LevelDebandingSettings";
import {
  ILevelEnhancedBloomField,
  ILevelLookField,
  LEVEL_ENHANCED_BLOOM_FIELDS,
  LEVEL_LOOK_BLOOM_GROUP,
  LEVEL_LOOK_FIELD_GROUPS,
} from "@/applications/level-viewer/lib/look/level-look-fields";
import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import { ERenderBloomMode, ERenderLightModel, RenderBloomMode, RenderLightModel } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderBloomMode,
  formatBloomStrength,
  RENDER_BLOOM_MODE_OPTIONS,
  RENDER_ENHANCED_BLOOM_LIMITS,
} from "@/core/render/lib/features/render-bloom-choices";
import { RENDER_LIGHT_MODEL_OPTIONS } from "@/core/render/lib/features/render-light-model-choices";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the list's option for values edited by hand stands for. */
const EDITED: string = "edited";

/** The looks offered whatever was saved, by their sources. */
const BUILT_IN_SOURCES: ReadonlyArray<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>> = [
  ELevelLookSource.GAME,
  ELevelLookSource.XRF,
  ELevelLookSource.ANOMALY,
  ELevelLookSource.OPENXRAY,
  ELevelLookSource.SETTINGS,
];

/** Each built-in look's name in the list. */
const BUILT_IN_LABELS: Readonly<Record<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>, string>> = {
  [ELevelLookSource.GAME]: "Game defaults",
  [ELevelLookSource.XRF]: "XRF",
  [ELevelLookSource.ANOMALY]: "Anomaly",
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
  const { settings, features } = useLevelRenderFeatures();
  const { set: setBloom, reset: resetBloom } = useLevelFeatureOverride("enhancedBloom");

  // Which bloom the view draws, and the enhanced bloom's strengths.
  const bloom: TRenderEnhancedBloomSettings = toLevelModeFeature("enhancedBloom", settings, features);
  const isEnhancedBloom: boolean = bloom.mode === ERenderBloomMode.ENHANCED;
  // What the view lights in: its own choice, else the settings'.
  const lightModel: RenderLightModel = features.lightModel ?? settings.lightModel;

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

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Look"}
      description={`Exposure, light and image as ${options.find((it) => it.value === picked)?.label ?? "set"}`}
      icon={<ExposureIcon />}
      isActive={picked !== ELevelLookSource.GAME || isEnhancedBloom || lightModel === ERenderLightModel.PHYSICAL}
    >
      <div className={"flex w-72 flex-col gap-3 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Look
        </Typography>

        <ChoiceListFormRow
          data-testid={"level-look-source"}
          label={"Source"}
          description={"Picks where the exposure, light scales, bloom and image corrections come from"}
          options={options}
          value={picked}
          filterFrom={options.length + 1}
          onChange={onPick}
        />

        <RenderValueChoice
          label={"Light"}
          options={RENDER_LIGHT_MODEL_OPTIONS}
          value={lightModel}
          onChange={(next: RenderLightModel) => viewService.setFeatures({ ...features, lightModel: next })}
        />

        <CheckboxFormRow
          label={"Auto exposure"}
          description={"Adapts the exposure to how bright the frame is; off, a fixed scale. Game setting: r2_tonemap."}
          isChecked={look.exposure.isEnabled}
          onChange={(isEnabled: boolean) => lookService.edit({ ...look, exposure: { ...look.exposure, isEnabled } })}
        />

        <RenderValueChoice
          label={"Bloom mode"}
          options={RENDER_BLOOM_MODE_OPTIONS}
          value={bloom.mode}
          onChange={(mode: RenderBloomMode) => setBloom({ mode })}
        />

        <p className={"text-xs text-text-secondary"}>{explainRenderBloomMode(bloom.mode)}</p>

        {isEnhancedBloom ? (
          <div className={"flex flex-col gap-1"}>
            <Typography variant={"caption"} className={"text-text-secondary"}>
              Enhanced bloom
            </Typography>

            {LEVEL_ENHANCED_BLOOM_FIELDS.map((field: ILevelEnhancedBloomField) => (
              <RenderValueSlider
                key={field.key}
                label={field.label}
                value={bloom[field.key]}
                {...RENDER_ENHANCED_BLOOM_LIMITS[field.key]}
                format={formatBloomStrength}
                onChange={(value: number) => setBloom({ [field.key]: value })}
              />
            ))}
          </div>
        ) : (
          <CheckboxFormRow
            label={"Bloom"}
            description={"Blurs the frame's bright parts over it. Game setting: phase_bloom."}
            isChecked={look.bloom.isEnabled}
            onChange={(isEnabled: boolean) => lookService.edit({ ...look, bloom: { ...look.bloom, isEnabled } })}
          />
        )}

        <Button size={"small"} aria-label={"Reset the bloom to the preset"} onClick={resetBloom}>
          Reset to the preset
        </Button>

        {LEVEL_LOOK_FIELD_GROUPS.filter((group) => !isEnhancedBloom || group.title !== LEVEL_LOOK_BLOOM_GROUP).map(
          (group) => (
            <div key={group.title} className={"flex flex-col gap-1"}>
              <Typography variant={"caption"} className={"text-text-secondary"}>
                {group.title}
              </Typography>

              {group.fields.map((field: ILevelLookField) => (
                <RenderValueSlider
                  key={field.label}
                  label={field.label}
                  value={field.read(look)}
                  min={field.min}
                  max={field.max}
                  step={field.step}
                  format={(value: number) => formatNumber(value, 2)}
                  onChange={(value: number) => lookService.edit(field.write(look, value))}
                />
              ))}
            </div>
          )
        )}

        <LevelDebandingSettings className={"flex flex-col gap-2"} />
      </div>
    </EditorPopoverAction>
  );
}
