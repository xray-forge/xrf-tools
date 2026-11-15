import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { IStringListFormRowProps, StringListFormRow } from "./StringListFormRow";

const SPAWN_SECTIONS: Array<string> = ["stalker_bandit_master", "m_bloodsucker_e", "af_medusa"];

function toValuesArgs(values: Array<string>): Partial<IStringListFormRowProps> {
  return { values };
}

function StringListFormRowStory(args: IStringListFormRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValuesArgs);

  return <StringListFormRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Form/StringListFormRow",
  component: StringListFormRow,
  render: StringListFormRowStory,
  decorators: [withReadingColumn],
  argTypes: {
    values: { control: false },
  },
  args: {
    label: "Skipped sections",
    description: "Spawn sections the packer leaves out of all.spawn, in the order given.",
    values: SPAWN_SECTIONS,
    addLabel: "Add section",
    emptyLabel: "Every section is packed.",
    placeholder: "Section name",
    onChange: fn(),
  },
} satisfies Meta<typeof StringListFormRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Entries edit, add and remove in place; an empty one keeps its row. */
export const Entries: TStory = {};

export const Empty: TStory = {
  args: { values: [] },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
