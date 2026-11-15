import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement, useCallback } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Button } from "#/material/button";
import { useArgChange } from "#/testing/stories";

import { ConfirmDialog, IConfirmDialogProps } from "./ConfirmDialog";

function toClosedArgs(): Partial<IConfirmDialogProps> {
  return { isOpen: false };
}

function ConfirmDialogStory(args: IConfirmDialogProps): ReactElement {
  const [, updateArgs] = useArgs<IConfirmDialogProps>();
  const onConfirm = useArgChange(args.onConfirm, toClosedArgs);
  const onClose = useArgChange(args.onClose, toClosedArgs);

  const onOpen = useCallback(() => updateArgs({ isOpen: true }), [updateArgs]);

  return (
    <>
      <Button variant={"outlined"} onClick={onOpen}>
        Open dialog
      </Button>

      <ConfirmDialog {...args} onConfirm={onConfirm} onClose={onClose} />
    </>
  );
}

const meta = {
  title: "Components/Dialog/ConfirmDialog",
  component: ConfirmDialog,
  render: ConfirmDialogStory,
  args: {
    isOpen: true,
    title: "Unpack archives",
    description: "Writes 38,113 files under the output directory. Files already there are kept.",
    confirmLabel: "Unpack",
    onConfirm: fn(),
    onClose: fn(),
  },
} satisfies Meta<typeof ConfirmDialog>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

/** Overwrites or deletes: the confirming button takes the error colour, and cancel keeps the focus. */
export const Destructive: TStory = {
  args: {
    title: "Overwrite output",
    description: "Replaces 1,204 files in the output directory. This cannot be undone.",
    confirmLabel: "Overwrite",
    isDestructive: true,
  },
};

export const ConfirmDisabled: TStory = {
  args: { isConfirmDisabled: true },
};
