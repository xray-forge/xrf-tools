import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement, ReactNode } from "react";

import { FoundationSection } from "#/testing/stories";

import { getHeaderGlossImage, getWashImage, getWellFill } from "./surface";
import { ColorScheme, SURFACE, TEXT } from "./tokens";

type TSurfaceLevel = keyof typeof SURFACE;

interface IElevationPaintLevel {
  level: TSurfaceLevel;
  /** Whether the level carries the wash: the two planes do, what floats over them does not. */
  isWashed: boolean;
}

const PAINT_LEVELS: ReadonlyArray<IElevationPaintLevel> = [
  { level: "content", isWashed: true },
  { level: "frame", isWashed: true },
  { level: "overlay", isWashed: false },
];

const SCHEMES: ReadonlyArray<ColorScheme> = ["dark", "light"];

interface IElevationLevelProps {
  name: string;
  note: string;
  className: string;
  children?: ReactNode;
}

function ElevationLevel({ name, note, className, children }: IElevationLevelProps): ReactElement {
  return (
    <div className={`flex flex-col gap-2 p-4 ${className}`}>
      <div>
        <code className={"font-monospace text-xs"}>{name}</code>
        <p className={"text-xs text-text-secondary"}>{note}</p>
      </div>

      {children}
    </div>
  );
}

interface IElevationSampleProps {
  name: string;
  note: string;
  className: string;
}

function ElevationSample({ name, note, className }: IElevationSampleProps): ReactElement {
  return (
    <div className={`flex w-64 flex-col gap-1 border border-divider p-3 ${className}`}>
      <code className={"font-monospace text-xs"}>{name}</code>
      <p className={"text-xs text-text-secondary"}>{note}</p>
    </div>
  );
}

interface IElevationSchemePaintProps {
  scheme: ColorScheme;
}

/** The scheme-dependent paints from `surface.ts` over each level of one scheme, whatever the toolbar says. */
function ElevationSchemePaint({ scheme }: IElevationSchemePaintProps): ReactElement {
  return (
    <div className={"flex flex-col gap-2"} style={{ color: TEXT.primary[scheme] }}>
      <span className={"text-xs text-text-secondary"}>{scheme}</span>

      {PAINT_LEVELS.map(({ level, isWashed }: IElevationPaintLevel) => (
        <div key={level} className={"grid grid-cols-3 gap-2"}>
          <div
            className={"flex h-16 items-end p-1.5 text-xs"}
            style={{
              backgroundColor: SURFACE[level][scheme],
              backgroundImage: isWashed ? getWashImage(scheme) : "none",
            }}
          >
            {isWashed ? `${level} + wash` : level}
          </div>
          <div className={"flex h-16 items-end p-1.5 text-xs"} style={{ backgroundColor: SURFACE[level][scheme] }}>
            <div className={"w-full p-1.5"} style={{ backgroundColor: getWellFill(scheme) }}>
              well
            </div>
          </div>
          <div
            className={"flex h-16 items-end p-1.5 text-xs"}
            style={{ backgroundColor: SURFACE[level][scheme], backgroundImage: getHeaderGlossImage(scheme) }}
          >
            gloss
          </div>
        </div>
      ))}
    </div>
  );
}

const meta = {
  title: "Foundations/Elevation",
} satisfies Meta;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Levels: TStory = {
  render: () => (
    <FoundationSection
      title={"Levels"}
      description={"The three levels as the window composes them: the frame, the inset reading plane, and an overlay."}
    >
      <ElevationLevel
        name={"surface-frame"}
        note={"The window's furniture: caption, rails, status bar and both docked panel slots."}
        className={"surface-frame"}
      >
        <ElevationLevel
          name={"surface-content"}
          note={"The reading plane, inset with a divider edge and the slight surface corner."}
          className={"rounded-surface border border-divider surface-content"}
        >
          <div className={"flex flex-wrap items-start gap-4"}>
            <ElevationSample
              name={"bg-surface-overlay"}
              note={"What floats: menus, popovers, dialogs, MUI's paper."}
              className={"rounded-surface bg-surface-overlay shadow-[var(--xrf-shadow-overlay)]"}
            />
            <ElevationSample
              name={"bg-well"}
              note={"A recess holding input or a nested listing, always bordered."}
              className={"rounded-control bg-well"}
            />
          </div>

          <div className={"flex min-h-header items-center border-y border-divider header-band px-3"}>
            <code className={"font-monospace text-xs"}>header-band</code>
          </div>
        </ElevationLevel>
      </ElevationLevel>
    </FoundationSection>
  ),
};

export const SchemePaints: TStory = {
  render: () => (
    <FoundationSection
      title={"Scheme paints"}
      description={"The wash, the well and the header gloss over each level, in both schemes at once."}
    >
      <div className={"grid grid-cols-2 gap-6"}>
        {SCHEMES.map((scheme: ColorScheme) => (
          <ElevationSchemePaint key={scheme} scheme={scheme} />
        ))}
      </div>
    </FoundationSection>
  ),
};
