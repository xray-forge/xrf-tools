import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { EditorPanelProperty } from "#/editor/panel";
import { withListingPane } from "#/testing/stories";

import { ReadingColumn } from "./ReadingColumn";

const ROWS: ReactElement = (
  <>
    <EditorPanelProperty label={"Version"} value={"18"} isInline />
    <EditorPanelProperty label={"Geometry"} value={"levels\\l01_escape\\level.geom"} isMonospace isInline />
    <EditorPanelProperty label={"Vertex buffers"} value={"12"} caption={"One per vertex format"} isInline />
    <EditorPanelProperty label={"Index buffers"} value={"4"} isInline />
    <EditorPanelProperty label={"Sectors"} value={"38"} caption={"Portals join them"} isInline />
  </>
);

const meta = {
  title: "Components/Layout/ReadingColumn",
  component: ReadingColumn,
  decorators: [withListingPane],
  args: {
    children: ROWS,
  },
} satisfies Meta<typeof ReadingColumn>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Fills its pane, scrolls from the top, and centres a column no wider than the reading width. */
export const Default: TStory = {};
