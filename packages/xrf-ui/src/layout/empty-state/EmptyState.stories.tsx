import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { Button } from "#/button";
import { SearchOffIcon } from "#/icons/search-off";
import { withListingPane, withViewportPane } from "#/testing/stories";

import { EmptyState } from "./EmptyState";

const meta = {
  title: "Components/Layout/EmptyState",
  component: EmptyState,
  decorators: [withListingPane],
  args: {
    title: "No file open",
    description: "Double-click a file in the tree to read it here.",
  },
} satisfies Meta<typeof EmptyState>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

/** A dead end the surface can undo offers the way out under the description. */
export const WithAction: TStory = {
  args: {
    title: "Nothing matches",
    description: "No spawn object on l02_garbage has a section starting with 'm_controller'.",
    icon: <SearchOffIcon />,
    action: (
      <Button variant={"outlined"} onClick={fn()}>
        Clear filter
      </Button>
    ),
  },
};

/** In a viewport's inks, over its backdrop. */
export const OnViewport: TStory = {
  decorators: [withViewportPane],
  args: {
    title: "No level open",
    description: "Pick a level from a game root to fly it.",
    isOnViewport: true,
  },
};
