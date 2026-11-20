import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { ArchiveIcon } from "#/icons/archive";
import { withReadingCard } from "#/testing/stories";

import { DialogHeader } from "./DialogHeader";

const meta = {
  title: "Components/Dialog/DialogHeader",
  component: DialogHeader,
  decorators: [withReadingCard],
  args: {
    title: "Pack gamedata",
    titleId: "pack-gamedata-title",
    icon: <ArchiveIcon fontSize={"small"} />,
    onClose: fn(),
  },
} satisfies Meta<typeof DialogHeader>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A decorative icon, the heading the dialog is labelled by, and a named close action. */
export const Default: TStory = {};

/** Dismissal already offered by the dialog's actions, so the heading carries none. */
export const WithoutClose: TStory = {
  args: { icon: undefined, onClose: undefined },
};

/** A long title wraps anywhere rather than pushing the close action out. */
export const LongTitle: TStory = {
  args: { title: "Replace textures\\act\\act_stalker_bandit_bump#.dds in resources.db1" },
};
