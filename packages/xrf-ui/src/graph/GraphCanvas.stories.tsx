import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withListingPane } from "#/testing/stories";

import { layoutGraphNodes } from "./graph-layout";
import { TGraphEdge, TGraphNode } from "./graph.types";
import { GraphCanvas } from "./GraphCanvas";

const DIALOG_LINES: ReadonlyArray<[string, string]> = [
  ["start", "Got a job for you, stalker."],
  ["ask", "What kind of job?"],
  ["refuse", "Not interested."],
  ["details", "Bring me the documents from the Agroprom labs."],
  ["accept", "Deal."],
  ["bye", "Then get lost."],
];

const EDGES: Array<TGraphEdge> = [
  { id: "start-ask", source: "start", target: "ask" },
  { id: "start-refuse", source: "start", target: "refuse" },
  { id: "ask-details", source: "ask", target: "details" },
  { id: "details-accept", source: "details", target: "accept" },
  { id: "refuse-bye", source: "refuse", target: "bye" },
];

const NODES: Array<TGraphNode> = layoutGraphNodes(
  DIALOG_LINES.map(([id, label]) => ({ id, position: { x: 0, y: 0 }, data: { label } })),
  EDGES,
  { direction: "TB" }
);

const meta = {
  title: "Components/Graph/GraphCanvas",
  component: GraphCanvas,
  decorators: [withListingPane],
  argTypes: {
    nodes: { control: false },
    edges: { control: false },
  },
  args: {
    nodes: NODES,
    edges: EDGES,
    onNodesChange: fn(),
    onEdgesChange: fn(),
    onConnect: fn(),
    onSelectionChange: fn(),
  },
} satisfies Meta<typeof GraphCanvas>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A conversation laid out top to bottom by `layoutGraphNodes`, in the scheme the toolbar picks. */
export const Dialog: TStory = {};

/** A read-only surface still draws its handles, and refuses new links. */
export const ReadOnly: TStory = {
  args: { isConnectable: false },
};
