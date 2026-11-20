import type { Meta, StoryObj } from "@storybook/react-vite";

import { Link } from ".";

const meta = {
  title: "Components/Typography/Link",
  component: Link,
  argTypes: {
    underline: { control: "inline-radio", options: ["none", "hover", "always"] },
    variant: { control: "select", options: ["body1", "body2", "caption"] },
  },
  args: {
    href: "#",
    variant: "body2",
    children: "Open the repository",
  },
} satisfies Meta<typeof Link>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const UnderlineOnHover: TStory = {
  args: { underline: "hover" },
};
