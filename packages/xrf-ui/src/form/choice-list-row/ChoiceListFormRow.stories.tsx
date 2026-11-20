import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { IChoiceListFormRowOption } from "./choice-list-form-row-option";
import { ChoiceListFormRow, IChoiceListFormRowProps } from "./ChoiceListFormRow";

const LEVELS: ReadonlyArray<IChoiceListFormRowOption<string>> = [
  {
    value: "l01_escape",
    label: "Cordon",
    section: "The Zone's south",
    description: "The rookie village and the rail bridge",
  },
  { value: "l02_garbage", label: "Garbage", section: "The Zone's south", description: "The scrapyard and the hangar" },
  {
    value: "l03_agroprom",
    label: "Agroprom",
    section: "The Zone's south",
    description: "The institute and its tunnels",
  },
  { value: "l04_darkvalley", label: "Dark Valley", section: "The Zone's south", description: "The Freedom outpost" },
  { value: "l05_bar", label: "Bar", section: "The Zone's centre", description: "The 100 Rads and the Arena" },
  {
    value: "l06_rostok",
    label: "Wild Territory",
    section: "The Zone's centre",
    description: "The Rostok factory yard",
  },
  {
    value: "l07_military",
    label: "Army Warehouses",
    section: "The Zone's north",
    description: "The Duty-Freedom front",
  },
  {
    value: "l08_yantar",
    label: "Yantar",
    section: "The Zone's north",
    description: "The lake and the scientists' bunker",
  },
  { value: "l10_radar", label: "Red Forest radar", section: "The Zone's north", description: "The Brain Scorcher" },
  { value: "l11_pripyat", label: "Pripyat", section: "The Zone's north", description: "The city before the plant" },
  {
    value: "l12_stancia",
    label: "Chernobyl NPP",
    section: "The plant",
    description: "The sarcophagus",
    isDisabled: true,
  },
];

function toValueArgs(value: string): Partial<IChoiceListFormRowProps<string>> {
  return { value };
}

function ChoiceListFormRowStory(args: IChoiceListFormRowProps<string>): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <ChoiceListFormRow<string> {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/ChoiceListFormRow",
  component: ChoiceListFormRow<string>,
  render: ChoiceListFormRowStory,
  decorators: [withReadingColumn],
  argTypes: {
    options: { control: false },
  },
  args: {
    label: "Start level",
    description: "Where a new game spawns the actor.",
    options: LEVELS,
    value: "l01_escape",
    onChange: fn(),
  },
} satisfies Meta<IChoiceListFormRowProps<string>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Enough options for a filter, under their sections' headings, the chosen one described under the list. */
export const Levels: TStory = {};

/** Below `filterFrom` the list stands alone: a filter would be furniture. */
export const Few: TStory = {
  args: { options: LEVELS.slice(0, 4) },
};

export const Optional: TStory = {
  args: { isRequired: false },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
