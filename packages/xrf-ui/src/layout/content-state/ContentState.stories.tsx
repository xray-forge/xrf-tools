import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { withListingPane } from "#/testing/stories";
import { Typography } from "#/typography";

import { ContentState, IContentStateProps } from "./ContentState";

const HEADER: string = "version 10, 1,842 objects, graph guid 7c3a…e91f";

function renderHeader(value: string): ReactElement {
  return (
    <Typography className={"p-4"} variant={"body2"}>
      {value}
    </Typography>
  );
}

const meta = {
  title: "Components/Layout/ContentState",
  component: ContentState<string>,
  decorators: [withListingPane],
  args: {
    value: HEADER,
    loadingLabel: "Reading spawn chunk…",
    errorTitle: "Could not read this chunk",
    emptyTitle: "Nothing to show",
    emptyDescription: "Open a spawn file to read its chunks.",
    children: renderHeader,
  },
} satisfies Meta<IContentStateProps<string>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The value, drawn by the caller. */
export const Content: TStory = {};

/** A value held through a reload stays on screen; progress shows only while there is none. */
export const Loading: TStory = {
  args: { value: null, isLoading: true },
};

/** A failure replaces everything else, a held value included. */
export const Failed: TStory = {
  args: { error: new Error("all.spawn ends 312 bytes into its graph chunk."), onRetry: fn() },
};

export const Empty: TStory = {
  args: { value: null },
};
