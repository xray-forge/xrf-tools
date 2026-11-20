import type { Meta, StoryObj } from "@storybook/react-vite";
import { Nullable } from "@xrf/types";
import { ComponentProps, ReactElement, SyntheticEvent } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { Autocomplete, TextField } from ".";

type TAutocompleteProps = ComponentProps<typeof Autocomplete<string>>;

const SECTIONS: ReadonlyArray<string> = [
  "actor",
  "m_bloodsucker_e",
  "m_dog_e",
  "stalker",
  "wpn_ak74",
  "wpn_pm",
  "inventory_box",
];

function renderSectionInput(params: Parameters<TAutocompleteProps["renderInput"]>[0]): ReactElement {
  return <TextField {...params} label={"Section"} />;
}

function toValueArgs(_event: SyntheticEvent, value: Nullable<string>): Partial<TAutocompleteProps> {
  return { value };
}

function AutocompleteStory(args: TAutocompleteProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <Autocomplete<string> {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Input/Autocomplete",
  component: Autocomplete<string>,
  render: AutocompleteStory,
  decorators: [withReadingColumn],
  argTypes: {
    options: { control: false },
  },
  args: {
    options: SECTIONS,
    value: "stalker",
    renderInput: renderSectionInput,
    onChange: fn(),
  },
} satisfies Meta<TAutocompleteProps>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const Empty: TStory = {
  args: { value: null },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
