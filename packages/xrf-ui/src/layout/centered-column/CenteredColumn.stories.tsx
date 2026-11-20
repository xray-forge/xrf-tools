import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { XrfMark } from "#/brand";
import { withListingPane } from "#/testing/stories";
import { Typography } from "#/typography";

import { CenteredColumn } from "./CenteredColumn";

const OPENING: ReactElement = (
  <>
    <XrfMark core={"optimized"} size={48} />
    <Typography variant={"subtitle1"}>Opening gamedata</Typography>
    <Typography className={"text-text-secondary"} variant={"body2"}>
      D:\Games\S.T.A.L.K.E.R. - Call of Pripyat\gamedata
    </Typography>
  </>
);

const meta = {
  title: "Components/Layout/CenteredColumn",
  component: CenteredColumn,
  decorators: [withListingPane],
  args: {
    children: OPENING,
  },
} satisfies Meta<typeof CenteredColumn>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Fills what it is given and centres its children on both axes, a small gap between them. */
export const Default: TStory = {};
