import type { Meta, StoryObj } from "@storybook/react-vite";

import { CircularProgress } from ".";

const meta = {
  title: "Components/Feedback/CircularProgress",
  component: CircularProgress,
  argTypes: {
    variant: { control: "inline-radio", options: ["indeterminate", "determinate"] },
    value: { control: { type: "range", min: 0, max: 100 } },
  },
  args: {
    "aria-label": "Reading archives",
    size: 32,
  },
} satisfies Meta<typeof CircularProgress>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Indeterminate: TStory = {};

export const Determinate: TStory = {
  args: { variant: "determinate", value: 64 },
};
