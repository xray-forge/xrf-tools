import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { INumberFieldProps, NumberField } from "./NumberField";

function toCommittedArgs(value: number): Partial<INumberFieldProps> {
  return { value };
}

function toTenths(value: number): string {
  return String(Math.round(value * 10) / 10);
}

function NumberFieldStory(args: INumberFieldProps): ReactElement {
  const onCommit = useArgChange(args.onCommit, toCommittedArgs);

  return <NumberField {...args} onCommit={onCommit} />;
}

const meta = {
  title: "Components/Form/NumberField",
  component: NumberField,
  render: NumberFieldStory,
  decorators: [withReadingColumn],
  args: {
    label: "Least x",
    value: -412.5,
    onCommit: fn(),
  },
} satisfies Meta<typeof NumberField>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Clear the field or type a lone `-`: the text is held and marked until it is a number again. */
export const Field: TStory = {};

/** A value set from outside, rounded to a tenth as a level map's distances are. */
export const Rounded: TStory = {
  args: { value: 12.3456, format: toTenths },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
