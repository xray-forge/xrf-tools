import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { withReadingColumn } from "#/testing/stories";

import { Collapse } from ".";

const COLLAPSED_BODY: ReactElement = (
  <div className={"rounded-control border border-divider bg-well p-3 text-sm"}>
    Folded away and unfolded again by `in`, height animated, nothing else.
  </div>
);

const meta = {
  title: "Components/Disclosure/Collapse",
  component: Collapse,
  decorators: [withReadingColumn],
  args: {
    in: true,
    children: COLLAPSED_BODY,
  },
} satisfies Meta<typeof Collapse>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Toggle `in` in the Controls panel to watch it fold. */
export const Open: TStory = {};

export const Closed: TStory = {
  args: { in: false },
};
