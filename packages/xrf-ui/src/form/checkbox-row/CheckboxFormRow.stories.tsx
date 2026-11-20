import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { CheckboxFormRow, ICheckboxFormRowProps } from "./CheckboxFormRow";

/** A choice made by hand is a definite one, so it also clears the indeterminate state. */
function toCheckedArgs(isChecked: boolean): Partial<ICheckboxFormRowProps> {
  return { isChecked, isIndeterminate: false };
}

function CheckboxFormRowStory(args: ICheckboxFormRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <CheckboxFormRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/CheckboxFormRow",
  component: CheckboxFormRow,
  render: CheckboxFormRowStory,
  decorators: [withReadingColumn],
  args: {
    label: "Unpack into subfolders",
    description: "Each archive is written under a folder named after it.",
    isChecked: true,
    onChange: fn(),
  },
} satisfies Meta<typeof CheckboxFormRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Checked: TStory = {};

export const Clear: TStory = {
  args: { isChecked: false },
};

export const Indeterminate: TStory = {
  args: { isChecked: false, isIndeterminate: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
