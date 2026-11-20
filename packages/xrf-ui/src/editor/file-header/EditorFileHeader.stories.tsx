import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { EditorIconAction } from "#/editor/icon-action";
import { ContentCopyIcon } from "#/icons/content-copy";
import { MapIcon } from "#/icons/map";
import { withReadingCard } from "#/testing/stories";

import { EditorFileHeader } from "./EditorFileHeader";

const meta = {
  title: "Components/Editor/EditorFileHeader",
  component: EditorFileHeader,
  decorators: [withReadingCard],
  args: {
    name: "config\\weapons\\w_ak74.ltx",
    caption: "4.2 KB",
    onClose: fn(),
  },
} satisfies Meta<typeof EditorFileHeader>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The whole path as the engine names it, a caption beside it and the close action last. */
export const Default: TStory = {};

export const WithActions: TStory = {
  args: {
    actions: (
      <EditorIconAction
        label={"Copy path"}
        description={"Copy the file's path"}
        icon={<ContentCopyIcon />}
        onClick={fn()}
      />
    ),
  },
};

/** A subject that is not a file: its own glyph and a close action named for it. */
export const Level: TStory = {
  args: {
    name: "l01_escape",
    caption: "1,204 spawn objects",
    icon: <MapIcon className={"text-text-secondary"} fontSize={"small"} />,
    closeLabel: "Close level",
    closeDescription: "Clear the selection and close this level",
  },
};

/** A surface that opened one subject has nothing to return to, so no close action. */
export const WithoutClose: TStory = {
  args: { onClose: undefined },
};
