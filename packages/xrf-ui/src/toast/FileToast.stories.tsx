import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { VideocamIcon } from "#/icons/videocam";
import { withViewportPane } from "#/testing/stories";

import { FileToast } from "./FileToast";

const meta = {
  title: "Components/Toast/FileToast",
  component: FileToast,
  decorators: [withViewportPane],
  args: {
    file: "C:\\appdata\\level-viewer\\screenshots\\zaton-20261104-230005.png",
    title: "Screenshot saved",
    name: "zaton-20261104-230005.png",
    details: "1920 × 1080 · 2.1 MB",
    preview: <div className={"h-14 w-24 checkerboard"} />,
    opening: { label: "Open the screenshot", onOpen: fn() },
    onReveal: fn(),
    onCopyPath: fn(),
    onDelete: fn(),
  },
} satisfies Meta<typeof FileToast>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A screenshot, its thumbnail opening it whole. Delete asks for a second click. */
export const Screenshot: TStory = {};

/** A recording that dropped frames: an icon in the well and a warning in the details. */
export const WarnedRecording: TStory = {
  args: {
    file: "C:\\appdata\\level-viewer\\recordings\\zaton-20261104-231210.mp4",
    title: "Recording saved",
    name: "zaton-20261104-231210.mp4",
    details: "0:42 · 61.4 MB · 12 frames dropped",
    isWarned: true,
    preview: <VideocamIcon className={"text-text-secondary"} />,
    opening: undefined,
  },
};
