import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { withReadingColumn } from "#/testing/stories";
import { Typography } from "#/typography";

import { renderCodeSpans } from "./code-spans";

interface ICodeSpansStoryArgs {
  text: string;
}

function CodeSpansStory({ text }: ICodeSpansStoryArgs): ReactElement {
  return <Typography variant={"body2"}>{renderCodeSpans(text)}</Typography>;
}

const meta = {
  title: "Components/Code/CodeSpans",
  component: CodeSpansStory,
  decorators: [withReadingColumn],
  args: {
    text: "Hides the `wpn_scope` bone while `hud_mode` is off, as a help line names them.",
  },
} satisfies Meta<typeof CodeSpansStory>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Edit the text: each backticked span becomes inline code. */
export const Spans: TStory = {};

export const Plain: TStory = {
  args: { text: "Nothing marked up comes back as it went in." },
};
