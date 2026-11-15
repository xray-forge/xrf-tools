import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { ISliderReferenceSource, SliderReferenceContext } from "#/form/reference-slider";
import { useArgChange, withReadingColumn } from "#/testing/stories";

import { ISliderFormRowProps, SliderFormRow } from "./SliderFormRow";

/** A source answering one number for every place, the way the settings answer a popover's sliders. */
const PRESET_REFERENCE: ISliderReferenceSource = { label: "Base preset", resolve: () => 300 };

function toValueArgs(value: number): Partial<ISliderFormRowProps> {
  return { value };
}

function SliderFormRowStory(args: ISliderFormRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <SliderFormRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/SliderFormRow",
  component: SliderFormRow,
  render: SliderFormRowStory,
  decorators: [withReadingColumn],
  args: {
    label: "Draw distance",
    description: "How far geometry is drawn, in metres.",
    value: 450,
    min: 50,
    max: 1500,
    step: 10,
    format: (value: number) => `${value} m`,
    onChange: fn(),
  },
} satisfies Meta<typeof SliderFormRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

/** The recommended band on the rail, and the wasted range past the measured useful end drawn red. */
export const Recommended: TStory = {
  args: { recommended: [200, 800], wastedAbove: 1200, value: 1300 },
};

/** Measured by a reference source: its value notched on the track, and a reset back to it beside the readout. */
export const WithReference: TStory = {
  decorators: [
    (Story) => (
      <SliderReferenceContext.Provider value={PRESET_REFERENCE}>
        <Story />
      </SliderReferenceContext.Provider>
    ),
  ],
  args: { recommended: [200, 800], referencePath: ["render", "draw_distance"] },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
