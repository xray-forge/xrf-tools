import type { Meta, StoryObj } from "@storybook/react-vite";

import { withReadingColumn } from "#/testing/stories";

import { LinearProgress } from ".";

const meta = {
  title: "Components/Feedback/LinearProgress",
  component: LinearProgress,
  decorators: [withReadingColumn],
  argTypes: {
    variant: { control: "inline-radio", options: ["indeterminate", "determinate", "buffer"] },
    value: { control: { type: "range", min: 0, max: 100 } },
  },
  args: {
    "aria-label": "Unpacking",
  },
} satisfies Meta<typeof LinearProgress>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Indeterminate: TStory = {};

export const Determinate: TStory = {
  args: { variant: "determinate", value: 64 },
};
