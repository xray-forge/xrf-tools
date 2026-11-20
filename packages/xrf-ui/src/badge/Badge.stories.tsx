import type { Meta, StoryObj } from "@storybook/react-vite";

import { NotificationsIcon } from "#/icons/notifications";

import { Badge } from ".";

const meta = {
  title: "Components/Badge/Badge",
  component: Badge,
  argTypes: {
    color: { control: "select", options: ["default", "primary", "secondary", "error", "warning", "success"] },
    variant: { control: "inline-radio", options: ["standard", "dot"] },
  },
  args: {
    badgeContent: 4,
    color: "primary",
    children: <NotificationsIcon />,
  },
} satisfies Meta<typeof Badge>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Count: TStory = {};

/** Past `max` the count reads as a cap, `99+`. */
export const Capped: TStory = {
  args: { badgeContent: 120 },
};

export const Dot: TStory = {
  args: { variant: "dot", color: "error" },
};
