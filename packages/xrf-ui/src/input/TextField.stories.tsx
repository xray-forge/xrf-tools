import type { Meta, StoryObj } from "@storybook/react-vite";
import { ChangeEvent, ComponentProps, ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { TextField } from ".";

type TTextFieldProps = ComponentProps<typeof TextField>;

function toValueArgs(event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>): Partial<TTextFieldProps> {
  return { value: event.target.value };
}

function TextFieldStory(args: TTextFieldProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <TextField {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Input/TextField",
  component: TextField,
  render: TextFieldStory,
  decorators: [withReadingColumn],
  argTypes: {
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    label: "Output directory",
    value: "gamedata",
    fullWidth: true,
    onChange: fn(),
  },
} satisfies Meta<typeof TextField>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A well: the field's fill is a recess on whatever level hosts it, and small is every field's size. */
export const Default: TStory = {};

export const WithHelperText: TStory = {
  args: { helperText: "Created beside the archives when it does not exist." },
};

export const Invalid: TStory = {
  args: { value: "", error: true, helperText: "Name the output directory." },
};

export const Multiline: TStory = {
  args: { label: "Notes", value: "First line\nSecond line", multiline: true, minRows: 3 },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
