import { default as ExposureIcon } from "@mui/icons-material/Exposure";
import { Button, TextField, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useState } from "react";

import { ELevelLookSource, ILevelLook, ILevelLookPreset } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { ChoiceListFormRow, IChoiceFormRowOption } from "@/core/ui/form";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

/** What a picked option of the list stands for: a look of its own source, a saved preset, or the values edited. */
const EDITED: string = "edited";
const PRESET_PREFIX: string = "preset:";

/** The looks offered whatever was saved, by their sources. */
const BUILT_IN_SOURCES: ReadonlyArray<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>> = [
  ELevelLookSource.GAME,
  ELevelLookSource.ANOMALY,
  ELevelLookSource.OPENXRAY,
  ELevelLookSource.SETTINGS,
];

/** Each built-in look's name in the list. */
const BUILT_IN_LABELS: Readonly<Record<Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>, string>> = {
  [ELevelLookSource.GAME]: "Game defaults",
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
 * How the level is exposed, lit and corrected: the game's console defaults, the settings', or a look of the viewer's
 * own, from a preset saved here or edited by hand.
 */
export function LevelLookAction({
  "data-testid": dataTestId = "level-look-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const lookService: LevelLookService = useInjection(LevelLookService);
  const [name, setName] = useState<string>("");

  const { choice, look, game } = lookService;
  const picked: string =
    choice.source === ELevelLookSource.CUSTOM
      ? choice.preset
        ? `${PRESET_PREFIX}${choice.preset}`
        : EDITED
      : choice.source;
  const options: Array<IChoiceFormRowOption<string>> = [
    ...BUILT_IN_SOURCES.map((source) => ({
      label:
        source === ELevelLookSource.GAME && !game
          ? `${BUILT_IN_LABELS[source]} (none shipped, the settings')`
          : BUILT_IN_LABELS[source],
      value: source as string,
    })),
    ...choice.presets.map((preset: ILevelLookPreset) => ({
      label: preset.name,
      value: `${PRESET_PREFIX}${preset.name}`,
    })),
    ...(picked === EDITED ? [{ label: "Edited by hand", value: EDITED }] : []),
  ];

  function onPick(value: string): void {
    const source = BUILT_IN_SOURCES.find((it) => it === value);

    if (source) {
      lookService.setSource(source);
    } else if (value.startsWith(PRESET_PREFIX)) {
      lookService.pickPreset(value.slice(PRESET_PREFIX.length));
    }
  }

  function onSave(): void {
    lookService.savePreset(name);
    setName("");
  }

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Look"}
      description={`Exposure, light and image as ${options.find((it) => it.value === picked)?.label ?? "set"}`}
      icon={<ExposureIcon />}
      isActive={picked !== ELevelLookSource.GAME}
    >
      <div className={"flex w-72 flex-col gap-3 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Look
        </Typography>

        <ChoiceListFormRow
          data-testid={"level-look-preset"}
          label={"Preset"}
          description={"Where the exposure, the light scales and the image corrections come from"}
          options={options}
          value={picked}
          filterFrom={options.length + 1}
          onChange={onPick}
        />

        <CheckboxFormRow
          label={"Adapt exposure"}
          description={"`r2_tonemap`: off, the frame is tonemapped at a fixed scale"}
          isChecked={look.exposure.isEnabled}
          onChange={(isEnabled: boolean) => lookService.edit({ ...look, exposure: { ...look.exposure, isEnabled } })}
        />

        {FIELD_GROUPS.map((group) => (
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

        <div className={"flex items-center gap-2"}>
          <TextField
            className={"min-w-0 grow"}
            size={"small"}
            placeholder={"Preset name"}
            value={name}
            slotProps={{ htmlInput: { "aria-label": "Preset name" } }}
            onChange={(event: ChangeEvent<HTMLInputElement>) => setName(event.target.value)}
          />
          <Button className={"min-w-0 shrink-0"} size={"small"} disabled={!name.trim()} onClick={onSave}>
            Save
          </Button>
        </div>

        {choice.source === ELevelLookSource.CUSTOM && choice.preset ? (
          <Button size={"small"} onClick={() => lookService.deletePreset(choice.preset ?? "")}>
            {`Delete "${choice.preset}"`}
          </Button>
        ) : null}
      </div>
    </EditorPopoverAction>
  );
}
