import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { FocusEvent, KeyboardEvent, ReactElement, useCallback, useId, useMemo, useState } from "react";

import { ButtonBase } from "#/button";
import {
  ColorPicker,
  fromPickedColor,
  IPickedColor,
  rescaleColor,
  TColorChannels,
  toColorIntensity,
  toPickedColor,
  useOpenColorField,
} from "#/color";
import { BaseComponentProps } from "#/element-types";
import { InputBase } from "#/input";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

interface IEditorPopoverVectorDraft {
  index: number;
  text: string;
}

/** What each component of a colour is called, the cover or the strength last. */
const COLOR_COMPONENTS: ReadonlyArray<string> = ["red", "green", "blue", "alpha"];

/** The draft index the intensity is typed under, past every component's. */
const INTENSITY_DRAFT: number = -1;

/** What each component of any other vector is called. */
const VECTOR_COMPONENTS: ReadonlyArray<string> = ["x", "y", "z", "w"];

function toSwatch(value: ReadonlyArray<number>): string {
  const [red = 0, green = 0, blue = 0] = value.map((it: number) => Math.round(saturate(it) * 255));

  return `rgb(${red}, ${green}, ${blue})`;
}

export interface IEditorPopoverVectorRowProps extends BaseComponentProps {
  label: string;
  /** What it holds, read on hovering the label and by a screen reader, such as the config key it is written under. */
  description?: string;
  /** Two to four components. */
  value: ReadonlyArray<number>;
  /**
   * Whether it is a colour: shown by a swatch of its first three components, clamped, which opens a picker of them, and
   * typed with an intensity that scales them past one.
   */
  isColor?: boolean;
  onChange: (value: Array<number>) => void;
}

/**
 * A few numbers in a stacked row of a toolbar popover or panel, each typed as it is held, unclamped; the owner is told
 * once one is let go. Too many to share a row with the label, they stand under it, a colour's swatch first.
 */
export function EditorPopoverVectorRow({
  "data-testid": dataTestId = "editor-popover-vector-row",
  id,
  className,
  label,
  description,
  value,
  isColor = false,
  onChange,
}: IEditorPopoverVectorRowProps): ReactElement {
  // Only the component being typed in is a draft, so a value the owner moves meanwhile moves every other.
  const [draft, setDraft] = useState<Nullable<IEditorPopoverVectorDraft>>(null);
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
    <EditorPopoverRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      isStacked
    >
      {(control: IEditorPopoverRowControlProps) => (
        <div className={"flex w-full min-w-0 flex-col"}>
          <div
            {...control}
            role={"group"}
            className={"grid w-full items-center gap-1"}
            style={{
              gridTemplateColumns: `${isColor ? "auto " : ""}repeat(${value.length + (isColor ? 1 : 0)}, minmax(0, 1fr))`,
            }}
          >
            {isColor ? (
              <ButtonBase
                className={"h-4 w-6 rounded-control border border-solid border-divider"}
                style={{ backgroundColor: toSwatch(value) }}
                aria-label={`Pick ${label}`}
                aria-expanded={isPicking}
                onClick={togglePicking}
              />
            ) : null}

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
      )}
    </EditorPopoverRow>
  );
}
