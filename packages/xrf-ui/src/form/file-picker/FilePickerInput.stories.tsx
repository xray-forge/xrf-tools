import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { FilePickerInput, IFilePickerInputProps } from "./FilePickerInput";
import { IPathFieldRecents } from "./path-field-recents";

const NOW: number = Date.UTC(2026, 10, 15, 12);

const RECENTS: IPathFieldRecents = {
  records: [
    { path: "D:\\Games\\Anomaly\\gamedata", at: NOW - 3_600_000 },
    { path: "D:\\Games\\Stalker COP\\gamedata", at: NOW - 86_400_000 * 3 },
  ],
  pick: fn(),
  forget: fn(),
};

function toValueArgs(value: string): Partial<IFilePickerInputProps> {
  return { value };
}

function toClearedArgs(): Partial<IFilePickerInputProps> {
  return { value: null };
}

function FilePickerInputStory(args: IFilePickerInputProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);
  const onClear = useArgChange(args.onClear, toClearedArgs);

  return (
    <FilePickerInput
      {...args}
      onChange={args.onChange ? onChange : undefined}
      onClear={args.onClear ? onClear : undefined}
    />
  );
}

const meta = {
  title: "Components/Form/FilePickerInput",
  component: FilePickerInput,
  render: FilePickerInputStory,
  decorators: [withReadingColumn],
  args: {
    value: "D:\\Games\\Anomaly\\gamedata",
    onSelect: fn(),
    onChange: fn(),
    onClear: fn(),
  },
} satisfies Meta<typeof FilePickerInput>;

export default meta;

type TStory = StoryObj<typeof meta>;

/**
 * The field alone: the folder button reports `onSelect`, where the application opens the native dialog, which a
 * story cannot.
 */
export const Default: TStory = {};

export const Empty: TStory = {
  args: { value: null },
};

/** Paths given before, offered from the history button at the field's end. */
export const WithRecents: TStory = {
  args: { recents: RECENTS },
};

/** Without `onChange` the field only reports what the dialog returned. */
export const DialogOnly: TStory = {
  args: { onChange: undefined },
};

/** As a popover or panel row's field: that row's height, its buttons as small as the field. */
export const Compact: TStory = {
  args: { isCompact: true },
};

export const Invalid: TStory = {
  args: { isInvalid: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
