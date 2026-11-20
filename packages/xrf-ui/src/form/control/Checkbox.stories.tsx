import type { Meta, StoryObj } from "@storybook/react-vite";
import { ChangeEvent, ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange } from "#/testing/stories";

import { Checkbox } from ".";

type TCheckboxProps = ComponentProps<typeof Checkbox>;

/** A box ticked by hand is a definite choice, so it also clears the indeterminate state. */
function toCheckedArgs(_event: ChangeEvent<HTMLInputElement>, checked: boolean): Partial<TCheckboxProps> {
  return { checked, indeterminate: false };
}

function CheckboxStory(args: TCheckboxProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <Checkbox {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/Checkbox",
  component: Checkbox,
  render: CheckboxStory,
  argTypes: {
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    slotProps: { input: { "aria-label": "Verify checksums" } },
    checked: true,
    size: "small",
    onChange: fn(),
  },
} satisfies Meta<typeof Checkbox>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Checked: TStory = {};

export const Clear: TStory = {
  args: { checked: false },
};

export const Indeterminate: TStory = {
  args: { checked: false, indeterminate: true },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
