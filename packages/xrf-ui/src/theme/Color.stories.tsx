import type { Meta, StoryObj } from "@storybook/react-vite";
import { isRecord } from "@xrf/types";
import { Fragment, ReactElement } from "react";

import { FoundationSection } from "#/testing/stories";

import {
  ACCENT,
  CATALOG_ACCENT,
  ColorScheme,
  DIVIDER,
  OVERLAY_BORDER,
  STATE_TONE,
  STATUS,
  SURFACE,
  TEXT,
  VIEWPORT,
  VIEWPORT_INK,
} from "./tokens";

type TSchemeColor = Readonly<Record<ColorScheme, string>>;

interface IColorToken {
  /** Its path in `tokens.ts`. */
  name: string;
  /** A pair for a scheme-dependent token, or one value both schemes share. */
  value: TSchemeColor | string;
}

/** The Tailwind utilities the tokens reach components by. */
const PALETTE_CLASSES: ReadonlyArray<string> = [
  "bg-primary",
  "bg-secondary",
  "bg-success",
  "bg-warning",
  "bg-error",
  "bg-text-primary",
  "bg-text-secondary",
  "bg-divider",
  "bg-surface-content",
  "bg-surface-frame",
  "bg-surface-overlay",
];

function isSchemeColor(value: unknown): value is TSchemeColor {
  return isRecord(value) && typeof value.light === "string" && typeof value.dark === "string";
}

/** Walks a token object down to its colours, naming each by its path in `tokens.ts`. */
function toColorTokens(name: string, value: unknown): Array<IColorToken> {
  if (typeof value === "string" || isSchemeColor(value)) {
    return [{ name, value }];
  }

  if (isRecord(value)) {
    return Object.entries(value).flatMap(([key, child]) => toColorTokens(`${name}.${key}`, child));
  }

  return [];
}

function toSchemeValue(token: IColorToken, scheme: ColorScheme): string {
  return typeof token.value === "string" ? token.value : token.value[scheme];
}

interface IColorSwatchProps {
  scheme: ColorScheme;
  color: string;
}

/** A colour on its own scheme's reading plane, so each value is judged where it is used, whatever the toolbar says. */
function ColorSwatch({ scheme, color }: IColorSwatchProps): ReactElement {
  return (
    <div
      className={"flex items-center gap-2 rounded-control p-1.5"}
      style={{
        backgroundColor: SURFACE.content[scheme],
        color: TEXT.primary[scheme],
        boxShadow: `inset 0 0 0 1px ${DIVIDER[scheme]}`,
      }}
    >
      <span
        className={"size-6 shrink-0 rounded-control"}
        style={{ backgroundColor: color, boxShadow: `inset 0 0 0 1px ${DIVIDER[scheme]}` }}
      />
      <code className={"font-monospace text-xs"}>{color}</code>
    </div>
  );
}

interface IColorTokenTableProps {
  tokens: ReadonlyArray<IColorToken>;
}

function ColorTokenTable({ tokens }: IColorTokenTableProps): ReactElement {
  return (
    <div className={"grid grid-cols-[minmax(12rem,auto)_1fr_1fr] items-center gap-x-4 gap-y-1.5"}>
      <span className={"text-xs text-text-secondary"}>Token</span>
      <span className={"text-xs text-text-secondary"}>Dark</span>
      <span className={"text-xs text-text-secondary"}>Light</span>

      {tokens.map((token: IColorToken) => (
        <Fragment key={token.name}>
          <code className={"font-monospace text-xs"}>{token.name}</code>
          <ColorSwatch scheme={"dark"} color={toSchemeValue(token, "dark")} />
          <ColorSwatch scheme={"light"} color={toSchemeValue(token, "light")} />
        </Fragment>
      ))}
    </div>
  );
}

const meta = {
  title: "Foundations/Color",
} satisfies Meta;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Accents: TStory = {
  render: () => (
    <FoundationSection
      title={"Accents"}
      description={
        "All hue in the window comes from the accents; primary is the brand's amber, secondary the cool end."
      }
    >
      <ColorTokenTable tokens={toColorTokens("ACCENT", ACCENT)} />
    </FoundationSection>
  ),
};

export const Status: TStory = {
  render: () => (
    <FoundationSection
      title={"Status"}
      description={"Success, warning and error. Warning is yellow-gold so it never reads as the amber accent."}
    >
      <ColorTokenTable tokens={toColorTokens("STATUS", STATUS)} />
    </FoundationSection>
  ),
};

export const Neutrals: TStory = {
  render: () => (
    <FoundationSection
      title={"Neutrals"}
      description={"Surfaces, text, dividers and borders keep equal channels, so all chrome hue comes from the wash."}
    >
      <ColorTokenTable
        tokens={[
          ...toColorTokens("SURFACE", SURFACE),
          ...toColorTokens("TEXT", TEXT),
          ...toColorTokens("DIVIDER", DIVIDER),
          ...toColorTokens("OVERLAY_BORDER", OVERLAY_BORDER),
          ...toColorTokens("STATE_TONE", STATE_TONE),
        ]}
      />
    </FoundationSection>
  ),
};

export const CatalogAccents: TStory = {
  render: () => (
    <FoundationSection
      title={"Catalog accents"}
      description={"The launcher's per-group wayfinding hues. They tint icons, so they answer to the 3:1 tier."}
    >
      <ColorTokenTable tokens={toColorTokens("CATALOG_ACCENT", CATALOG_ACCENT)} />
    </FoundationSection>
  ),
};

export const Viewport: TStory = {
  render: () => (
    <FoundationSection
      title={"Viewport"}
      description={"The plane a picture is judged against, dark in both schemes, and the ink it carries."}
    >
      <ColorTokenTable
        tokens={[...toColorTokens("VIEWPORT", VIEWPORT), ...toColorTokens("VIEWPORT_INK", VIEWPORT_INK)]}
      />
    </FoundationSection>
  ),
};

/** The Tailwind names the tokens reach the application by, drawn in whichever scheme the toolbar picks. */
export const ThemePalette: TStory = {
  render: () => (
    <FoundationSection
      title={"Theme palette"}
      description={"The utilities components paint with, resolved through MUI's variables in the scheme shown."}
    >
      <div className={"flex flex-wrap gap-3"}>
        {PALETTE_CLASSES.map((className: string) => (
          <div key={className} className={"flex w-32 flex-col gap-1"}>
            <div className={`h-12 rounded-control border border-divider ${className}`} />
            <code className={"font-monospace text-xs"}>{className}</code>
          </div>
        ))}
      </div>
    </FoundationSection>
  ),
};
