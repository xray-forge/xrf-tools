import { default as ExposureIcon } from "@mui/icons-material/Exposure";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatNumber } from "@xrf/format";
import { ReactElement, useCallback } from "react";

import { ERenderBloomMode, RenderBloomMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderBloomMode,
  formatBloomStrength,
  RENDER_BLOOM_MODE_OPTIONS,
  RENDER_ENHANCED_BLOOM_LIMITS,
} from "@/core/render/lib/features/render-bloom-choices";
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

/** One number of the look a slider sets. */
interface ILookField {
  label: string;
  min: number;
  max: number;
  step: number;
  read: (look: ILevelLook) => number;
  write: (look: ILevelLook, value: number) => ILevelLook;
}

/** The title of the group of the engine bloom's sliders, which the enhanced bloom's replace. */
const BLOOM_GROUP: string = "Bloom";

/** One of the enhanced bloom's strengths a slider sets. */
interface IEnhancedBloomField {
  key: Exclude<keyof TRenderEnhancedBloomSettings, "mode">;
  label: string;
}

/** The enhanced bloom's sliders, in the order they are offered. */
const ENHANCED_BLOOM_FIELDS: ReadonlyArray<IEnhancedBloomField> = [
  { key: "threshold", label: "Threshold" },
  { key: "exposure", label: "Brightness" },
  { key: "blur", label: "Blur" },
  { key: "vibrance", label: "Vibrance" },
  { key: "sky", label: "Sky" },
];

/** The sliders, by the group they sit under. */
const FIELD_GROUPS: ReadonlyArray<{ title: string; fields: ReadonlyArray<ILookField> }> = [
  {
    fields: [
      {
        label: "Amount",
        max: 1,
        min: 0,
        read: (look) => look.exposure.amount,
        step: 0.01,
        write: (look, amount) => ({ ...look, exposure: { ...look.exposure, amount } }),
      },
      {
        label: "Middle gray",
        max: 2,
        min: 0,
        read: (look) => look.exposure.middleGray,
        step: 0.05,
        write: (look, middleGray) => ({ ...look, exposure: { ...look.exposure, middleGray } }),
      },
      {
        label: "Low luminance",
        max: 1,
        min: 0.0001,
        read: (look) => look.exposure.lowLuminance,
        step: 0.01,
        write: (look, lowLuminance) => ({ ...look, exposure: { ...look.exposure, lowLuminance } }),
      },
      {
        label: "Adaptation",
        max: 10,
        min: 0.01,
        read: (look) => look.exposure.adaptation,
        step: 0.01,
        write: (look, adaptation) => ({ ...look, exposure: { ...look.exposure, adaptation } }),
      },
    ],
    title: "Tonemap",
  },
  {
    fields: [
      {
        label: "Sun",
        max: 4,
        min: 0,
        read: (look) => look.lightScales.sun,
        step: 0.05,
        write: (look, sun) => ({ ...look, lightScales: { ...look.lightScales, sun } }),
      },
      {
        label: "Hemisphere",
        max: 4,
        min: 0,
        read: (look) => look.lightScales.hemi,
        step: 0.05,
        write: (look, hemi) => ({ ...look, lightScales: { ...look.lightScales, hemi } }),
      },
      {
        label: "Ambient",
        max: 4,
        min: 0,
        read: (look) => look.lightScales.ambient,
        step: 0.05,
        write: (look, ambient) => ({ ...look, lightScales: { ...look.lightScales, ambient } }),
      },
    ],
    title: "Light scales",
  },
  {
    fields: [
      {
        label: "Threshold",
        max: 1,
        min: 0,
        read: (look) => look.bloom.threshold,
        step: 0.01,
        write: (look, threshold) => ({ ...look, bloom: { ...look.bloom, threshold } }),
      },
      {
        label: "Radius",
        max: 7,
        min: 1,
        read: (look) => look.bloom.radius,
        step: 0.1,
        write: (look, radius) => ({ ...look, bloom: { ...look.bloom, radius } }),
      },
      {
        label: "Strength",
        max: 2,
        min: 0.05,
        read: (look) => look.bloom.strength,
        step: 0.05,
        write: (look, strength) => ({ ...look, bloom: { ...look.bloom, strength } }),
      },
    ],
    title: BLOOM_GROUP,
  },
  {
    fields: [
      {
        label: "Exposure",
        max: 4,
        min: 0.5,
        read: (look) => look.corrections.exposure,
        step: 0.05,
        write: (look, exposure) => ({ ...look, corrections: { ...look.corrections, exposure } }),
      },
      {
        label: "Gamma",
        max: 2.2,
        min: 0.5,
        read: (look) => look.corrections.gamma,
        step: 0.05,
        write: (look, gamma) => ({ ...look, corrections: { ...look.corrections, gamma } }),
      },
      {
        label: "Saturation",
        max: 2,
        min: 0,
        read: (look) => look.corrections.saturation,
        step: 0.05,
        write: (look, saturation) => ({ ...look, corrections: { ...look.corrections, saturation } }),
      },
      ...(["Grading red", "Grading green", "Grading blue"] as const).map(
        (label: string, channel: number): ILookField => ({
          label,
          max: 1,
          min: 0,
          read: (look) => look.corrections.grading[channel] ?? 0,
          step: 0.01,
          write: (look, value) => {
            const grading: [number, number, number] = [...look.corrections.grading];

            grading[channel] = value;

            return { ...look, corrections: { ...look.corrections, grading } };
          },
        })
      ),
    ],
    title: "Image",
  },
];

/**
 * How the level is exposed, lit, bloomed and corrected: the game's console defaults, the settings', a built-in engine's,
 * or values edited by hand; and whether the engine's bloom draws or the enhanced one in its place.
 */
export function LevelLookAction({
  "data-testid": dataTestId = "level-look-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const lookService: LevelLookService = useInjection(LevelLookService);
  const { settings, features } = useLevelRenderFeatures();
  const { set: setBloom, reset: resetBloom } = useLevelFeatureOverride("enhancedBloom");

  // Which bloom the view draws, and the enhanced bloom's strengths.
  const bloom: TRenderEnhancedBloomSettings = toLevelModeFeature("enhancedBloom", settings, features);
  const isEnhancedBloom: boolean = bloom.mode === ERenderBloomMode.ENHANCED;

  const { choice, look, defaults } = lookService;
  const picked: string = choice.source === ELevelLookSource.CUSTOM ? EDITED : choice.source;
  const options: Array<IChoiceFormRowOption<string>> = [
    ...BUILT_IN_SOURCES.map((source) => ({
      label:
        source === ELevelLookSource.GAME && !defaults?.isShipped
          ? `${BUILT_IN_LABELS[source]} (none shipped, the engine's)`
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
      isActive={picked !== ELevelLookSource.GAME || isEnhancedBloom}
    >
      <div className={"flex w-72 flex-col gap-3 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Look
        </Typography>

        <ChoiceListFormRow
          data-testid={"level-look-source"}
          label={"Source"}
          description={"Where the exposure, the light scales, the bloom and the image corrections come from"}
          options={options}
          value={picked}
          filterFrom={options.length + 1}
          onChange={onPick}
        />

        <CheckboxFormRow
          label={"Adapt exposure"}
          description={"Adapts the exposure to how bright the frame is; off, a fixed scale. Game setting: r2_tonemap."}
          isChecked={look.exposure.isEnabled}
          onChange={(isEnabled: boolean) => lookService.edit({ ...look, exposure: { ...look.exposure, isEnabled } })}
        />

        <RenderValueChoice
          label={"Bloom drawn as"}
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

            {ENHANCED_BLOOM_FIELDS.map((field: IEnhancedBloomField) => (
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

        <Button size={"small"} onClick={resetBloom}>
          Reset to the preset
        </Button>

        {FIELD_GROUPS.filter((group) => !isEnhancedBloom || group.title !== BLOOM_GROUP).map((group) => (
          <div key={group.title} className={"flex flex-col gap-1"}>
            <Typography variant={"caption"} className={"text-text-secondary"}>
              {group.title}
            </Typography>

            {group.fields.map((field: ILookField) => (
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
        ))}
      </div>
    </EditorPopoverAction>
  );
}
