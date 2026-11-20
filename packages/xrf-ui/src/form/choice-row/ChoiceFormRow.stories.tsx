import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { IChoiceFormRowOption } from "./choice-form-row-option";
import { ChoiceFormRow, IChoiceFormRowProps } from "./ChoiceFormRow";

type TQuality = "low" | "medium" | "high" | "ultra";

const QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TQuality>> = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
  { value: "ultra", label: "Ultra", isDisabled: true },
];

function toValueArgs(value: TQuality): Partial<IChoiceFormRowProps<TQuality>> {
  return { value };
}

function ChoiceFormRowStory(args: IChoiceFormRowProps<TQuality>): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <ChoiceFormRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/ChoiceFormRow",
  component: ChoiceFormRow<TQuality>,
  render: ChoiceFormRowStory,
  decorators: [withReadingColumn],
  args: {
    label: "Texture quality",
    description: "How much of each texture's mip chain is loaded.",
    options: QUALITY_OPTIONS,
    value: "medium",
    onChange: fn(),
  },
} satisfies Meta<IChoiceFormRowProps<TQuality>>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const Optional: TStory = {
  args: { isRequired: false },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
