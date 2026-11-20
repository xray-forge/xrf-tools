import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { Slider, SliderProps } from ".";

const DISTANCE_MARKS: NonNullable<SliderProps["marks"]> = [
  { value: 200, label: "200 m" },
  { value: 800, label: "800 m" },
];

function toValueArgs(_event: Event, value: number | Array<number>): Partial<SliderProps> {
  return { value };
}

function SliderStory(args: SliderProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <Slider {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/Slider",
  component: Slider,
  render: SliderStory,
  decorators: [withReadingColumn],
  argTypes: {
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    "aria-label": "Draw distance",
    value: 450,
    min: 50,
    max: 1500,
    step: 10,
    size: "small",
    valueLabelDisplay: "auto",
    onChange: fn(),
  },
} satisfies Meta<typeof Slider>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The thumb's hit area held to 24px by the theme, so it stays inside a padded panel at either end. */
export const Default: TStory = {};

export const WithMarks: TStory = {
  args: { marks: DISTANCE_MARKS },
};

export const Range: TStory = {
  args: { value: [200, 800] },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
