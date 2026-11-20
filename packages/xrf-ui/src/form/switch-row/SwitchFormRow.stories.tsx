import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { ISwitchFormRowProps, SwitchFormRow } from "./SwitchFormRow";

function toCheckedArgs(isChecked: boolean): Partial<ISwitchFormRowProps> {
  return { isChecked };
}

function SwitchFormRowStory(args: ISwitchFormRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <SwitchFormRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/SwitchFormRow",
  component: SwitchFormRow,
  render: SwitchFormRowStory,
  decorators: [withReadingColumn],
  args: {
    label: "Verify after packing",
    description: "Reads every archive back and compares it with gamedata.",
    isChecked: true,
    onChange: fn(),
  },
} satisfies Meta<typeof SwitchFormRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const On: TStory = {};

export const Off: TStory = {
  args: { isChecked: false },
};

export const Optional: TStory = {
  args: { isRequired: false },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
