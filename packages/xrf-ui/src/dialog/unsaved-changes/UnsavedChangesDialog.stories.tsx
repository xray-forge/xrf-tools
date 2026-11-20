import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement, useCallback } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Button } from "#/button";
import { useArgChange } from "#/testing/stories";

import { IUnsavedChangesDialogProps, UnsavedChangesDialog } from "./UnsavedChangesDialog";

function toClosedArgs(): Partial<IUnsavedChangesDialogProps> {
  return { isOpen: false };
}

function UnsavedChangesDialogStory(args: IUnsavedChangesDialogProps): ReactElement {
  const [, updateArgs] = useArgs<IUnsavedChangesDialogProps>();
  const onSave = useArgChange(args.onSave ?? undefined, toClosedArgs);
  const onDiscard = useArgChange(args.onDiscard, toClosedArgs);
  const onClose = useArgChange(args.onClose, toClosedArgs);

  const onOpen = useCallback(() => updateArgs({ isOpen: true }), [updateArgs]);

  return (
    <>
      <Button variant={"outlined"} onClick={onOpen}>
        Open dialog
      </Button>

      <UnsavedChangesDialog {...args} onSave={args.onSave ? onSave : null} onDiscard={onDiscard} onClose={onClose} />
    </>
  );
}

const meta = {
  title: "Components/Dialog/UnsavedChangesDialog",
  component: UnsavedChangesDialog,
  render: UnsavedChangesDialogStory,
  args: {
    isOpen: true,
    description: "3 edited spawn objects on l01_escape are not written to all.spawn yet.",
    onDiscard: fn(),
    onClose: fn(),
  },
} satisfies Meta<typeof UnsavedChangesDialog>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Stay keeps the focus; discarding takes the error colour, saving the primary one. */
export const Default: TStory = {
  args: { onSave: fn() },
};

/** A save asked for from here holds every button until it answers. */
export const Saving: TStory = {
  args: { isSaving: true, onSave: fn() },
};

/** Where the work cannot be saved from here, only kept or thrown away. */
export const WithoutSave: TStory = {
  args: {
    title: "Close the translation?",
    description: "The edits to st_dialogs_escape.xml are held in memory only.",
    onSave: null,
  },
};
