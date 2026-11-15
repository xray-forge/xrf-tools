import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { FoundationSection } from "#/testing/stories";
import { MONOSPACE, MONOSPACE_CHARACTER_WIDTH } from "#/theme";
import { Typography } from "#/typography";

const VARIANTS = ["h5", "h6", "subtitle1", "subtitle2", "body1", "body2", "button", "caption", "overline"] as const;

type TTypographyVariant = (typeof VARIANTS)[number];

const SPECIMEN_TEXT: string = "Gamedata archive, 1,204 entries";

const RULER_TEXT: string = "meshes\\actors\\stalker_bandit\\";

/** How many characters the ruler measures, past the end of its text. */
const RULER_CHARACTERS: number = 40;

interface ITypographySpecimenProps {
  variant: TTypographyVariant;
}

function TypographySpecimen({ variant }: ITypographySpecimenProps): ReactElement {
  return (
    <>
      <code className={"font-monospace text-xs text-text-secondary"}>{variant}</code>
      <Typography variant={variant}>{SPECIMEN_TEXT}</Typography>
    </>
  );
}

const meta = {
  title: "Foundations/Typography",
} satisfies Meta;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Scale: TStory = {
  render: () => (
    <FoundationSection
      title={"Scale"}
      description={"The theme's variants, the ones the kit draws with; Segoe first, Roboto as the bundled fallback."}
    >
      <div className={"grid grid-cols-[8rem_1fr] items-baseline gap-x-6 gap-y-3"}>
        {VARIANTS.map((variant: TTypographyVariant) => (
          <TypographySpecimen key={variant} variant={variant} />
        ))}
      </div>
    </FoundationSection>
  ),
};

/** The ruler is `MONOSPACE_CHARACTER_WIDTH` per character, so it ends with the text when the face matches the token. */
export const Monospace: TStory = {
  render: () => (
    <FoundationSection
      title={"Monospace"}
      description={"Text compared by eye rather than read as prose: identifiers, engine paths, entry names."}
    >
      <dl className={"grid grid-cols-[10rem_1fr] gap-x-6 gap-y-1 text-xs"}>
        <dt className={"text-text-secondary"}>fontFamily</dt>
        <dd className={"font-monospace"}>{MONOSPACE.fontFamily}</dd>
        <dt className={"text-text-secondary"}>fontSize</dt>
        <dd className={"font-monospace"}>{MONOSPACE.fontSize}</dd>
        <dt className={"text-text-secondary"}>character width</dt>
        <dd className={"font-monospace"}>{MONOSPACE_CHARACTER_WIDTH}px</dd>
      </dl>

      <div className={"flex flex-col gap-1"}>
        <span className={"monospace whitespace-pre"}>{RULER_TEXT.padEnd(RULER_CHARACTERS, ".")}</span>
        <div className={"h-1 bg-primary"} style={{ width: RULER_CHARACTERS * MONOSPACE_CHARACTER_WIDTH }} />
      </div>
    </FoundationSection>
  ),
};
