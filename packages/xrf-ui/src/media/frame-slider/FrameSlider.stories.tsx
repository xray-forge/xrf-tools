import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { FrameSlider, IFrameSliderProps } from "./FrameSlider";

function toSeekArgs(frame: number): Partial<IFrameSliderProps> {
  return { frame };
}

function FrameSliderStory(args: IFrameSliderProps): ReactElement {
  const onSeek = useArgChange(args.onSeek, toSeekArgs);

  return <FrameSlider {...args} onSeek={onSeek} />;
}

const meta = {
  title: "Components/Media/FrameSlider",
  component: FrameSlider,
  render: FrameSliderStory,
  decorators: [withReadingColumn],
  args: {
    ariaLabel: "Motion frame",
    frameCount: 51,
    frame: 20,
    onSeek: fn(),
  },
} satisfies Meta<typeof FrameSlider>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Fifty-one frames, numbered zero to fifty. */
export const Motion: TStory = {};

/** Nothing posed, so nothing to move. */
export const Empty: TStory = {
  args: { frameCount: 0, frame: 0 },
};
