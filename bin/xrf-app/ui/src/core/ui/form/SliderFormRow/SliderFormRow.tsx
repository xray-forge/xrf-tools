import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback } from "react";

import { FormRow } from "@/core/ui/form/FormRow";
import { ISliderReference, ReferenceSlider, SliderReadout, useSliderReference } from "@/core/ui/form/ReferenceSlider";
import { IThrottledDraft, useThrottledDraft } from "@/lib/react/use-throttled-draft";

export interface ISliderFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside the slider, and to a screen reader. */
  format?: (value: number) => string;
  isDisabled?: boolean;
  /** The setting's place in the reference source about it, whose value is marked and reset to. */
  referencePath?: ReadonlyArray<string>;
  /** The range xrf recommends, drawn on the rail; none where the setting is not rated. */
  recommended?: readonly [number, number];
  /** Past it a higher value looks the same and only costs more, drawn red on the rail; none where not measured. */
  wastedAbove?: number;
  /** Told the value while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: number) => void;
}

/**
 * A labelled, controlled number within a range, its value read out beside it, measured by the reference source about
 * it and the range xrf recommends.
 */
export function SliderFormRow({
  "data-testid": dataTestId = "slider-form-row",
  id,
  className,
  label,
  description,
  value,
  min,
  max,
  step,
  format = String,
  isDisabled = false,
  referencePath,
  recommended,
  wastedAbove,
  onChange,
}: ISliderFormRowProps): ReactElement {
  const draft: IThrottledDraft = useThrottledDraft(value, step, onChange);
  const reference: Nullable<ISliderReference> = useSliderReference(referencePath);
  const referenceValue: Nullable<number> = reference?.value ?? null;

  const onReset = useCallback(() => {
    if (referenceValue !== null) {
      onChange(referenceValue);
    }
  }, [referenceValue, onChange]);

  return (
    <FormRow label={label} description={description} controlId={id}>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <div className={"flex items-center gap-4"}>
          <ReferenceSlider
            data-testid={dataTestId}
            id={controlId}
            className={className}
            size={"small"}
            value={draft.value}
            min={min}
            max={max}
            step={step}
            reference={referenceValue}
            recommended={recommended}
            wastedAbove={wastedAbove}
            disabled={isDisabled}
            getAriaValueText={format}
            slotProps={{ input: { "aria-labelledby": labelId, "aria-describedby": describedBy } }}
            onChange={draft.onChange}
            onChangeCommitted={draft.onChangeCommitted}
            onReset={referenceValue === null || isDisabled ? undefined : onReset}
          />
          <SliderReadout
            className={"w-24 shrink-0 justify-end"}
            value={draft.value}
            format={format}
            reference={reference}
            recommended={recommended}
            wastedAbove={wastedAbove}
            isDisabled={isDisabled}
            onReset={onChange}
          />
        </div>
      )}
    </FormRow>
  );
}
