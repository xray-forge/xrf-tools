import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ISliderReference, ReferenceSlider, SliderReadout, useSliderReference } from "@xrf/ui/form/reference-slider";
import { IThrottledDraft, useThrottledDraft } from "@xrf/ui/react/throttled-draft";
import { ReactElement, useCallback } from "react";

export interface IRenderValueSliderProps extends BaseComponentProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside its label and to a screen reader, which are the only places a unit is stated. */
  format: (value: number) => string;
  /** Shown on hovering its label: where the value comes from, such as the config key it is written under. */
  hint?: string;
  /** Whether it shows its value but cannot be moved, where the value has no effect. */
  isDisabled?: boolean;
  /** The setting's place in the reference source about it (`IRenderLimits` carries it), whose value is marked. */
  referencePath?: ReadonlyArray<string>;
  /** The range xrf recommends, drawn on the rail; none where the setting is not rated. */
  recommended?: readonly [number, number];
  /** Past it a higher value looks the same and only costs more, drawn red on the rail; none where not measured. */
  wastedAbove?: number;
  /** Told the value while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: number) => void;
}

/**
 * One numeric value a preview is driven by, with what it currently reads beside its name, measured by the reference
 * source about it and the range xrf recommends.
 */
export function RenderValueSlider({
  "data-testid": dataTestId = "render-value-slider",
  id,
  className,
  label,
  value,
  min,
  max,
  step,
  format,
  hint,
  isDisabled = false,
  referencePath,
  recommended,
  wastedAbove,
  onChange,
}: IRenderValueSliderProps): ReactElement {
  const draft: IThrottledDraft = useThrottledDraft(value, step, onChange);
  const reference: Nullable<ISliderReference> = useSliderReference(referencePath);
  const referenceValue: Nullable<number> = reference?.value ?? null;

  const onReset = useCallback(() => {
    if (referenceValue !== null) {
      onChange(referenceValue);
    }
  }, [referenceValue, onChange]);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={"flex items-baseline justify-between gap-2"}>
        <span className={"text-xs text-text-secondary"} title={hint}>
          {label}
        </span>
        <SliderReadout
          value={draft.value}
          format={format}
          reference={reference}
          recommended={recommended}
          wastedAbove={wastedAbove}
          isCompact
          isDisabled={isDisabled}
          onReset={onChange}
        />
      </div>

      <ReferenceSlider
        size={"small"}
        min={min}
        max={max}
        step={step}
        value={draft.value}
        reference={referenceValue}
        recommended={recommended}
        wastedAbove={wastedAbove}
        disabled={isDisabled}
        aria-label={label}
        getAriaValueText={format}
        onChange={draft.onChange}
        onChangeCommitted={draft.onChangeCommitted}
        onReset={referenceValue === null || isDisabled ? undefined : onReset}
      />
    </div>
  );
}
