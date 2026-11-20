import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement, useCallback } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { Button } from "#/button";
import { useArgChange } from "#/testing/stories";

import { Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from ".";

type TDialogProps = ComponentProps<typeof Dialog>;

const DIALOG_BODY: ReactElement = (
  <>
    <DialogTitle>Settings</DialogTitle>
    <DialogContent>
      <DialogContentText variant={"body2"}>
        The body takes the reading plane and its wash; the title and the actions keep the frame and its gloss.
      </DialogContentText>
    </DialogContent>
    <DialogActions>
      <Button size={"small"} onClick={fn()}>
        Cancel
      </Button>
      <Button size={"small"} variant={"contained"} onClick={fn()}>
        Save
      </Button>
    </DialogActions>
  </>
);

function toClosedArgs(): Partial<TDialogProps> {
  return { open: false };
}

function DialogStory(args: TDialogProps): ReactElement {
  const [, updateArgs] = useArgs<TDialogProps>();
  const onClose = useArgChange(args.onClose, toClosedArgs);

  const onOpen = useCallback(() => updateArgs({ open: true }), [updateArgs]);

  return (
    <>
      <Button variant={"outlined"} onClick={onOpen}>
        Open dialog
      </Button>

      <Dialog {...args} onClose={onClose} />
    </>
  );
}

const meta = {
  title: "Components/Dialog/Dialog",
  component: Dialog,
  render: DialogStory,
  argTypes: {
    maxWidth: { control: "inline-radio", options: ["xs", "sm", "md", "lg"] },
  },
  args: {
    open: true,
    maxWidth: "sm",
    fullWidth: true,
    children: DIALOG_BODY,
    onClose: fn(),
  },
} satisfies Meta<typeof Dialog>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The bare modal the kit's dialogs compose; `ConfirmDialog` is the one most surfaces want. */
export const Open: TStory = {};
