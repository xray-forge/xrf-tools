import { ButtonBase, InputBase } from "@mui/material";
import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { FocusEvent, KeyboardEvent, ReactElement, useCallback, useId, useMemo, useState } from "react";

import {
  fromPickedColor,
  IPickedColor,
  rescaleColor,
  TColorChannels,
  toColorIntensity,
  toPickedColor,
} from "@/core/ui/color/color-intensity";
import { ColorPicker } from "@/core/ui/color/ColorPicker";
import { useOpenColorField } from "@/core/ui/color/use-open-color-field";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What each component of a colour is called, the cover or the strength last. */
const COLOR_COMPONENTS: ReadonlyArray<string> = ["red", "green", "blue", "alpha"];

/** The draft index the intensity is typed under, past every component's. */
const INTENSITY_DRAFT: number = -1;

/** What each component of any other vector is called. */
const VECTOR_COMPONENTS: ReadonlyArray<string> = ["x", "y", "z", "w"];

interface ILevelWeatherVectorFieldProps extends BaseComponentProps {
  /** The engine key it holds. */
  label: string;
  value: ReadonlyArray<number>;
  /**
   * Whether it is a colour: shown by a swatch of its first three components, clamped, which opens a picker of them, and
   * typed with an intensity that scales them past one.
   */
  isColor?: boolean;
  onChange: (value: Array<number>) => void;
}

interface ILevelWeatherVectorDraft {
  index: number;
  text: string;
}

/**
 * A weather key of a few numbers, each typed as the engine holds it, unclamped; one is told once it is let go.
 */
export function LevelWeatherVectorField({
  "data-testid": dataTestId = "level-weather-vector-field",
  id,
  className,
  label,
  value,
  isColor = false,
  onChange,
}: ILevelWeatherVectorFieldProps): ReactElement {
  // Only the component being typed in is a draft, so the weather playing on moves every other.
  const [draft, setDraft] = useState<Nullable<ILevelWeatherVectorDraft>>(null);
  const [isPicking, togglePicking] = useOpenColorField(useId());
  const names: ReadonlyArray<string> = isColor ? COLOR_COMPONENTS : VECTOR_COMPONENTS;
  const channels: TColorChannels = useMemo(() => [value[0] ?? 0, value[1] ?? 0, value[2] ?? 0], [value]);
  const intensity: number = toColorIntensity(channels);

  // A colour's first three components, set whole; any after them kept.
  const onChangeChannels = useCallback(
    (next: TColorChannels) => onChange(value.map((it: number, at: number) => (at < 3 ? next[at] : it))),
    [value, onChange]
  );

  const onCommitIntensity = useCallback(
    (text: string) => {
      const parsed: number = Number(text);

      setDraft(null);

      if (text.trim() && Number.isFinite(parsed) && parsed > 0 && parsed !== intensity) {
        onChangeChannels(rescaleColor(channels, intensity, parsed));
      }
    },
    [channels, intensity, onChangeChannels]
  );

  const onCommit = useCallback(
    (index: number, text: string) => {
      const parsed: number = Number(text);

      setDraft(null);

      if (text.trim() && Number.isFinite(parsed) && parsed !== value[index]) {
        onChange(value.map((it: number, at: number) => (at === index ? parsed : it)));
      }
    },
    [value, onChange]
  );

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={"flex items-center justify-between gap-2"}>
        <span className={"font-mono text-xs text-text-secondary"}>{label}</span>
        {isColor ? (
          <ButtonBase
            className={"h-3 w-6 rounded-control border border-solid border-divider"}
            style={{ backgroundColor: toSwatch(value) }}
            aria-label={`Pick ${label}`}
            aria-expanded={isPicking}
            onClick={togglePicking}
          />
        ) : null}
      </div>

      <div
        className={"mt-1 grid gap-1"}
        style={{ gridTemplateColumns: `repeat(${value.length + (isColor ? 1 : 0)}, minmax(0, 1fr))` }}
      >
        {value.map((component: number, index: number) => (
          <InputBase
            key={names[index]}
            className={"rounded-control bg-action-hover px-1 font-mono text-xs"}
            value={draft?.index === index ? draft.text : String(Number(component.toFixed(6)))}
            inputProps={{ "aria-label": `${label} ${names[index]}`, inputMode: "decimal" }}
            onFocus={(event: FocusEvent<HTMLInputElement>) => setDraft({ index, text: event.target.value })}
            onChange={(event) => setDraft({ index, text: event.target.value })}
            onBlur={(event: FocusEvent<HTMLInputElement>) => onCommit(index, event.target.value)}
            onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
              if (event.key === "Enter") {
                event.currentTarget.blur();
              }
            }}
          />
        ))}

        {isColor ? (
          <InputBase
            className={"rounded-control bg-action-hover px-1 font-mono text-xs"}
            value={draft?.index === INTENSITY_DRAFT ? draft.text : String(Number(intensity.toFixed(3)))}
            inputProps={{ "aria-label": `${label} intensity`, inputMode: "decimal", title: "Intensity" }}
            onFocus={(event: FocusEvent<HTMLInputElement>) =>
              setDraft({ index: INTENSITY_DRAFT, text: event.target.value })
            }
            onChange={(event) => setDraft({ index: INTENSITY_DRAFT, text: event.target.value })}
            onBlur={(event: FocusEvent<HTMLInputElement>) => onCommitIntensity(event.target.value)}
            onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
              if (event.key === "Enter") {
                event.currentTarget.blur();
              }
            }}
          />
        ) : null}
      </div>

      {isColor && isPicking ? (
        <ColorPicker
          className={"mt-2"}
          value={toPickedColor(channels, intensity)}
          onChange={(picked: IPickedColor) => onChangeChannels(fromPickedColor(picked, intensity))}
        />
      ) : null}
    </div>
  );
}

function toSwatch(value: ReadonlyArray<number>): string {
  const [red = 0, green = 0, blue = 0] = value.map((it: number) => Math.round(saturate(it) * 255));

  return `rgb(${red}, ${green}, ${blue})`;
}
