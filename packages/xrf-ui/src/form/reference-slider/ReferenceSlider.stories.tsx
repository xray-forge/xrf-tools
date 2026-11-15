import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement, useCallback } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { IReferenceSliderProps, ReferenceSlider } from "./ReferenceSlider";
import { SliderReadout } from "./SliderReadout";

const REFERENCE_LABEL: string = "Base preset";

function formatMetres(value: number): string {
  return `${value} m`;
}

function toDraggedArgs(_event: Event, value: number | Array<number>): Partial<IReferenceSliderProps> {
  return { value: Array.isArray(value) ? value[0] : value };
}

/** A double click returns to the reference, which the slider reports without a value. */
function toReferenceArgs({ reference, value }: IReferenceSliderProps): Partial<IReferenceSliderProps> {
  return { value: reference ?? value };
}

/** The slider with its readout beside it, as `SliderFormRow` and the popover rows lay them out. */
function ReferenceSliderStory(args: IReferenceSliderProps): ReactElement {
  const onChange = useArgChange(args.onChange, toDraggedArgs);
  const onReset = useArgChange(args.onReset, toReferenceArgs);

  /** The readout hands back the reference it returns to, which the args already hold. */
  const onReadoutReset = useCallback(() => onReset(), [onReset]);

  return (
    <div className={"flex items-center gap-4"}>
      <ReferenceSlider {...args} onChange={onChange} onReset={args.onReset ? onReset : undefined} />
      <SliderReadout
        className={"w-24 shrink-0 justify-end"}
        value={args.value}
        format={formatMetres}
        reference={typeof args.reference === "number" ? { label: REFERENCE_LABEL, value: args.reference } : null}
        recommended={args.recommended}
        wastedAbove={args.wastedAbove}
        isDisabled={args.disabled}
        onReset={onReadoutReset}
      />
    </div>
  );
}

const meta = {
  title: "Components/Form/ReferenceSlider",
  component: ReferenceSlider,
  render: ReferenceSliderStory,
  decorators: [withReadingColumn],
  args: {
    "aria-label": "Draw distance",
    size: "small",
    value: 450,
    min: 50,
    max: 1500,
    step: 10,
    reference: 300,
    recommended: [200, 800],
    onChange: fn(),
    onReset: fn(),
  },
} satisfies Meta<typeof ReferenceSlider>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The reference notched on the track, the recommended band on the rail, and a reset to it beside the readout. */
export const Measured: TStory = {};

/** Outside the recommended range: the slider and the readout take the warning colour, which says why on hover. */
export const Outside: TStory = {
  args: { value: 1000 },
};

/** Past the measured useful end, drawn red on the rail and painted as an error. */
export const Wasted: TStory = {
  args: { value: 1300, wastedAbove: 1200 },
};

/** At the reference the reset has nothing to return to and steps aside. */
export const AtReference: TStory = {
  args: { value: 300 },
};

/** Nothing measured and nothing rated: a plain slider and its value. */
export const Unmeasured: TStory = {
  args: { reference: undefined, recommended: undefined, onReset: undefined },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
