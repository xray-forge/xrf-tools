import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, MouseEvent, ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange } from "#/testing/stories";

import { ToggleButton, ToggleButtonGroup } from ".";

type TToggleButtonGroupProps = ComponentProps<typeof ToggleButtonGroup>;

/** An array rather than a fragment: the group reads its buttons as its direct children. */
const QUALITY_BUTTONS: ReadonlyArray<ReactElement> = [
  <ToggleButton key={"low"} value={"low"}>
    Low
  </ToggleButton>,
  <ToggleButton key={"medium"} value={"medium"}>
    Medium
  </ToggleButton>,
  <ToggleButton key={"high"} value={"high"}>
    High
  </ToggleButton>,
  <ToggleButton key={"ultra"} value={"ultra"} disabled>
    Ultra
  </ToggleButton>,
];

function toValueArgs(_event: MouseEvent<HTMLElement>, value: unknown): Partial<TToggleButtonGroupProps> {
  return { value };
}

function ToggleButtonStory(args: TToggleButtonGroupProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <ToggleButtonGroup {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Button/ToggleButton",
  component: ToggleButtonGroup,
  render: ToggleButtonStory,
  argTypes: {
    color: { control: "select", options: ["standard", "primary", "secondary"] },
    size: { control: "inline-radio", options: ["small", "medium", "large"] },
  },
  args: {
    "aria-label": "Texture quality",
    children: QUALITY_BUTTONS,
    value: "medium",
    exclusive: true,
    color: "primary",
    size: "small",
    onChange: fn(),
  },
} satisfies Meta<typeof ToggleButtonGroup>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** One choice of several, as `ChoiceFormRow` draws it. */
export const Exclusive: TStory = {};

export const Standard: TStory = {
  args: { color: "standard" },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
