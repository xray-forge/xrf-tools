import { IThrottledDraft, useThrottledDraft } from "@xrf/react/throttled-draft";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { BaseComponentProps } from "#/element-types";
import { ISliderReference, ReferenceSlider, SliderReadout, useSliderReference } from "#/form/reference-slider";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

export interface IEditorPopoverSliderRowProps extends BaseComponentProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside its label and to a screen reader, which are the only places a unit is stated. */
  format: (value: number) => string;
  /** What it does, read on hovering its label. */
  description?: string;
  /** Read on hovering its label after what it does: where the value comes from, such as the config key it is under. */
  hint?: string;
  /** Whether it shows its value but cannot be moved, where the value has no effect. */
  isDisabled?: boolean;
  /** The setting's place in the nearest `SliderReferenceContext` source, whose value is marked. */
  referencePath?: ReadonlyArray<string>;
  /** The recommended range, drawn on the rail; none where the setting is not rated. */
  recommended?: readonly [number, number];
  /** Past it a higher value looks the same and only costs more, drawn red on the rail; none where not measured. */
  wastedAbove?: number;
  /** Told the value while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: number) => void;
}

/**
 * One numeric value in a single row of a toolbar popover: the label in the column every row shares, the slider beside
 * it and what it reads, measured by the reference source about it and the recommended range.
 */
export function EditorPopoverSliderRow({
  "data-testid": dataTestId = "editor-popover-slider-row",
  id,
  className,
  label,
  value,
  min,
  max,
  step,
  format,
  description,
  hint,
  isDisabled = false,
  referencePath,
  recommended,
  wastedAbove,
  onChange,
}: IEditorPopoverSliderRowProps): ReactElement {
  const draft: IThrottledDraft = useThrottledDraft(value, step, onChange);
  const reference: Nullable<ISliderReference> = useSliderReference(referencePath);
  const referenceValue: Nullable<number> = reference?.value ?? null;

  const onReset = useCallback(() => {
    if (referenceValue !== null) {
      onChange(referenceValue);
    }
  }, [referenceValue, onChange]);

  return (
    <EditorPopoverRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={[description, hint].filter(Boolean).join(" ") || undefined}
      isDisabled={isDisabled}
    >
      {(control: IEditorPopoverRowControlProps) => (
        <>
          <ReferenceSlider
            className={"min-w-0 flex-1"}
            size={"small"}
            min={min}
            max={max}
            step={step}
            value={draft.value}
            reference={referenceValue}
            recommended={recommended}
            wastedAbove={wastedAbove}
            disabled={isDisabled}
            getAriaValueText={format}
            slotProps={{ input: control }}
            onChange={draft.onChange}
            onChangeCommitted={draft.onChangeCommitted}
            onReset={referenceValue === null || isDisabled ? undefined : onReset}
          />

          <SliderReadout
            className={"ml-2 w-popover-readout shrink-0 justify-end"}
            value={draft.value}
            format={format}
            reference={reference}
            recommended={recommended}
            wastedAbove={wastedAbove}
            isCompact={true}
            isDisabled={isDisabled}
            onReset={onChange}
          />
        </>
      )}
    </EditorPopoverRow>
  );
}
