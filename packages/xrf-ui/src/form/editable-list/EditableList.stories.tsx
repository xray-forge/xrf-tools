import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { TextField } from "#/input";
import { withReadingColumn } from "#/testing/stories";

import { EditableList } from "./EditableList";
import { EditableListItem } from "./EditableListItem";

const EXCLUDED_ARCHIVES: ReadonlyArray<string> = ["resources.db0", "sounds.db1", "levels\\l12_stancia.db"];

/** Uncontrolled fields: the list only lays its rows out, and `StringListFormRow` is what owns their values. */
const ROWS: Array<ReactElement> = EXCLUDED_ARCHIVES.map((archive: string, index: number) => (
  <EditableListItem key={archive} removeLabel={`Remove ${archive}`} onRemove={fn()}>
    <TextField
      size={"small"}
      fullWidth
      defaultValue={archive}
      slotProps={{ htmlInput: { "aria-label": `Excluded archive ${index + 1}` } }}
    />
  </EditableListItem>
));

const DISABLED_ROWS: Array<ReactElement> = EXCLUDED_ARCHIVES.map((archive: string) => (
  <EditableListItem key={archive} removeLabel={`Remove ${archive}`} isDisabled onRemove={fn()}>
    <TextField size={"small"} fullWidth defaultValue={archive} disabled />
  </EditableListItem>
));

const meta = {
  title: "Components/Form/EditableList",
  component: EditableList,
  decorators: [withReadingColumn],
  argTypes: {
    children: { control: false },
  },
  args: {
    addLabel: "Add archive",
    emptyLabel: "Every archive is unpacked.",
    children: ROWS,
    onAdd: fn(),
  },
} satisfies Meta<typeof EditableList>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Rows of fields, each with its remove action, and the add action under them. */
export const Rows: TStory = {};

export const Empty: TStory = {
  args: { children: [] },
};

/** Held while an operation runs: the remove actions say to wait for it. */
export const Disabled: TStory = {
  args: { children: DISABLED_ROWS, isDisabled: true },
};
