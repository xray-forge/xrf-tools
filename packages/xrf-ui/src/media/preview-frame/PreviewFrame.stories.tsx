import type { Meta, StoryObj } from "@storybook/react-vite";

import { withViewportPane } from "#/testing/stories";
import { Typography } from "#/typography";

import { PreviewFrame } from "./PreviewFrame";

const meta = {
  title: "Components/Media/PreviewFrame",
  component: PreviewFrame,
  decorators: [withViewportPane],
  args: {
    caption: "act_stalker_bandit.dds · 1024 × 1024 · DXT5",
    children: (
      <div className={"flex grow items-center justify-center"}>
        <Typography className={"text-viewport-text"} variant={"body2"}>
          The picture sits here.
        </Typography>
      </div>
    ),
  },
} satisfies Meta<typeof PreviewFrame>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The viewport backdrop, as behind a picture that is still being read. */
export const Backdrop: TStory = {};

/** The alpha checkerboard, as behind a picture with transparency. */
export const Checkered: TStory = {
  args: { isCheckered: true },
};
