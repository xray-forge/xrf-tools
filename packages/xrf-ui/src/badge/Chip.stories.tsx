import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { Chip } from ".";

const meta = {
  title: "Components/Badge/Chip",
  component: Chip,
  argTypes: {
    color: { control: "select", options: ["default", "primary", "secondary", "error", "warning", "success"] },
    variant: { control: "inline-radio", options: ["filled", "outlined"] },
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    label: "configs.db",
    size: "small",
  },
} satisfies Meta<typeof Chip>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const Outlined: TStory = {
  args: { variant: "outlined", color: "primary" },
};

export const Deletable: TStory = {
  args: { onDelete: fn() },
};

export const Clickable: TStory = {
  args: { onClick: fn() },
};

export const Disabled: TStory = {
  args: { disabled: true, onDelete: fn() },
};
