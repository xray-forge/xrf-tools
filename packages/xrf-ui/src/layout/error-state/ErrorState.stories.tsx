import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withListingPane, withViewportPane } from "#/testing/stories";

import { ErrorState } from "./ErrorState";

const meta = {
  title: "Components/Layout/ErrorState",
  component: ErrorState,
  decorators: [withListingPane],
  args: {
    title: "Could not read the archive",
    description: "resources.db1 ends 4,096 bytes before its file table says it should.",
    onRetry: fn(),
  },
} satisfies Meta<typeof ErrorState>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Announced as an alert, with a retry through the request's owner. */
export const Retry: TStory = {};

export const WithoutRetry: TStory = {
  args: { onRetry: undefined },
};

/** In a viewport's inks, over its backdrop, for a renderer that stopped drawing. */
export const OnViewport: TStory = {
  decorators: [withViewportPane],
  args: {
    title: "The renderer stopped",
    description: "The GPU device was lost while drawing l01_escape.",
    onRetry: undefined,
    isOnViewport: true,
  },
};
