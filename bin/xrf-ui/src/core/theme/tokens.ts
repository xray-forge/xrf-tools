/**
 * Color scheme modes the application offers.
 *
 * `system` is not a scheme of its own: `theme-init.ts` resolves it against the OS preference before
 * first paint, so it must stay in sync with what that script accepts.
 */
export const COLOR_SCHEME_MODES = ["light", "dark", "system"] as const;

export type ColorSchemeMode = (typeof COLOR_SCHEME_MODES)[number];

export const DEFAULT_COLOR_SCHEME_MODE: ColorSchemeMode = "dark";

// Desktop radii. Windows 11 keeps most controls at 4-8px; anything rounder reads as a touch target.
export const RADIUS = {
  sm: 4,
  md: 6,
  lg: 8,
} as const;

export const BADGE_FONT_SIZE = "0.625rem";

/**
 * The plane a picture is judged against, and the alpha checkerboard drawn behind it.
 */
export const VIEWPORT = {
  backdrop: "#353535",
  checkerboardDark: "#707070",
  checkerboardLight: "#808080",
  /** Side of one checkerboard square, in css pixels. */
  checkerboardSquare: 10,
} as const;

/**
 * Diagonal accent wash over the frame and the reading plane.
 */
export const WASH = {
  angle: "135deg",
  /** Cool end, at the gradient's origin. */
  secondary: { light: 0.05, dark: 0.05 },
  /** Warm end. */
  primary: { light: 0.05, dark: 0.03 },
} as const;

/** Formats a token share for `color-mix`, which takes a percentage rather than a fraction. */
export function toSharePercent(share: number): string {
  return `${share * 100}%`;
}

export const LAYOUT = {
  /**
   * The corner a framed surface turns: the workspace edge, a form card, and the first-paint skeleton that stands in
   * for both. Named once so the three cannot drift, which a reader would only ever notice as a jump on load.
   */
  surfaceRadius: RADIUS.md,
  /**
   * The band a surface names itself in: the file header over the body, and a panel's title row. One value, so the
   * three run as a single line across the window instead of three headers that each end somewhere else.
   */
  headerHeight: 40,
  /** The button plus the breathing room on either side of it; the stripe holds nothing wider. */
  railWidth: 36,
  /** Every control in either stripe. */
  railButtonSize: 28,
  /** The glyph inside one, sized to fill the button rather than to sit in padding. */
  railButtonIconSize: 20,
  /** Unread count contained within a rail button, including the capped `99+` label. */
  railBadgeHeight: 14,
  railBadgeMinWidth: 14,
  railBadgeFontSize: BADGE_FONT_SIZE,
  railBadgePaddingX: 3,
  /** Dense `MuiToolbar` and `MuiTab` still measure themselves against this; the window caption does not. */
  toolbarHeight: 40,
  statusBarHeight: 28,
  /**
   * The window's only top band: caption and the active application's toolbar in one row.
   */
  titleBarHeight: 36,
  windowControlWidth: 36,
  windowControlHeight: 36,
  /**
   * A slider hosted in a toolbar popover.
   */
  toolbarSliderWidth: 200,
  /** The motion picker, wide enough for a name like `norm_walk_fwd_1` without truncating it. */
  motionPickerWidth: 260,
  /** Frame counter beside the motion slider, sized so a four digit frame of a four digit total does not reflow it. */
  motionCounterWidth: 118,
  /** Labelled rows read left to right, held to a column rather than stretched across the window. */
  readingColumnWidth: 860,
  /** Label column of a labelled row in one, wide enough for `External normal map` without wrapping. */
  readingLabelWidth: 200,
} as const;

/**
 * Form controls, shared by every surface that renders one.
 *
 * MUI derives an input's height from its font size (a `1.4375em` line box plus padding), so a field that
 * shrinks its own type -- a monospace path, say -- renders shorter than the fields beside it. These pin the
 * box instead, and `creation.ts` applies them to every small input so font size and height stay independent.
 */
export const CONTROL = {
  /** Compact editor actions, both in the caption and beside content. */
  editorActionSize: 24,
  editorActionIconSize: 16,
  editorActionFontSize: "0.75rem",
  /** Every `size="small"` input, which is all of them: the theme makes `small` the default. */
  smallHeight: 38,
  /** MUI's own `sizeSmall` padding above and below the input; the line box is whatever height is left. */
  smallInputPaddingY: 8.5,
} as const;

/** Shared dialog spacing in theme spacing units; the header minimum is in pixels. */
export const DIALOG = {
  paddingX: 3,
  headerPaddingY: 1.5,
  contentPaddingY: 3,
  actionsPaddingY: 1.5,
  gap: 1,
  headerMinHeight: 56,
} as const;

/**
 * Explorer trees, shared by every surface that renders one.
 *
 * Sized against `LAYOUT.railButtonSize` rather than freely: a tree row sits in the same panels as the rail's controls,
 * and rows that disagree with them read as a different application.
 */
export const TREE = {
  /** Row height, dense enough that a mesh directory is scannable without becoming a hit-target problem. */
  rowHeight: 28,
  /** A panel row's own controls, one step below the severity icon so they do not read as part of the outcome. */
  actionIconSize: 16,
  /** Expand and collapse chevron column. */
  iconWidth: 18,
  iconSize: 17,
  iconGap: 4,
  /** Indent per nesting level, which is the chevron column plus its gap. */
  indent: 14,
} as const;

/**
 * Source listings, shared by every surface that renders lines with a gutter.
 *
 * `lineHeight` is a pixel count rather than a ratio because a virtualized listing positions rows by
 * arithmetic: where line 200,000 sits has to be known without laying the 199,999 above it out.
 */
export const CODE = {
  /** One line, sized so a screenful of {@link MONOSPACE} is dense without the rows touching. */
  lineHeight: 20,
  /** Padding on each side of the gutter's contents. */
  gutterPaddingX: 8,
  /** Gap between a gutter mark and the number beside it. */
  gutterGap: 4,
  /** Narrowest gutter, so a short file does not draw a different column from the file beside it. */
  minimumGutterDigits: 3,
  /** A gutter mark, one step below the number so it reads as an annotation rather than a digit. */
  markIconSize: 13,
  /** Space between the gutter and the first character of a line. */
  contentPaddingX: 10,
} as const;

/**
 * Side panels, same on both sides.
 */
export const PANEL = {
  defaultWidth: 300,
  minWidth: 200,
  maxWidth: 640,
  /** Share of the window every open panel may occupy together, so the content keeps the rest. */
  maxWidthRatio: 0.5,
  /** A panel row's own controls, one step below the severity icon so they do not read as part of the outcome. */
  actionIconSize: 16,
  /** Shared spacing for panel headings, sections, and stacked properties, in theme units. */
  contentPadding: 2,
  headerPaddingY: 1.5,
  sectionPaddingY: 1.5,
  sectionContentGap: 1,
  propertyPaddingY: 0.75,
  propertyValueGap: 0.25,
  contentLineHeight: 1.6,
} as const;

/**
 * Text compared by eye rather than read as prose: identifiers, engine paths, archive entry names, file positions.
 *
 * One definition, because the theme's `.monospace` class and the surfaces that style their own rows have to agree - a
 * path in a panel and the same path in a grid cell that disagree on size read as two different kinds of thing. The
 * size is smaller than `body2` on purpose: these strings are long, and a panel is narrow.
 */
export const MONOSPACE = {
  fontFamily: "'Cascadia Mono', 'Consolas', monospace",
  fontSize: "0.75rem",
} as const;

/**
 * Advance width of one {@link MONOSPACE} character, in pixels.
 */
export const MONOSPACE_CHARACTER_WIDTH: number = 7.25;

export const ACCENT = {
  // Light is `#ffa200` darkened along its own hue rather than desaturated, so the accent and the mark agree.
  primary: {
    main: { light: "#9c5a00", dark: "#f5aa4d" },
    contrastText: { light: "#ffffff", dark: "#332414" },
  },
  secondary: {
    main: { light: "#56649c", dark: "#8e9ada" },
    contrastText: { light: "#ffffff", dark: "#20253f" },
  },
} as const;

export const STATUS = {
  success: {
    main: { light: "#3d671e", dark: "#9ccc65" },
    contrastText: { light: "#ffffff", dark: "#18230e" },
  },
  // Yellow-gold, not amber: an amber-branded window washed in amber cannot also warn in amber.
  warning: {
    main: { light: "#6b5d00", dark: "#e8c547" },
    contrastText: { light: "#ffffff", dark: "#231e07" },
  },
  error: {
    main: { light: "#a33028", dark: "#ff958a" },
    contrastText: { light: "#ffffff", dark: "#310c09" },
  },
} as const;

/**
 * The three opaque planes; every surface in the window is exactly one of them.
 *
 * The reading plane keeps the most contrast available for text and the others step toward the text colour. Dark
 * expresses elevation as lightness and light as shadow, which is why `overlay` steps in one scheme and not the other.
 */
export const SURFACE = {
  content: { light: "#ffffff", dark: "#171717" },
  frame: { light: "#f2f2f2", dark: "#212121" },
  overlay: { light: "#ffffff", dark: "#2e2e2e" },
} as const;

/** The tone a neutral state scrim is mixed from: toward the text colour in either scheme. */
export const STATE_TONE = { light: "#000000", dark: "#ffffff" } as const;

/** The tone a well is mixed from. A control that is not elevated must not gain lightness, so it recedes in both. */
export const RECESS_TONE: string = "#000000";

/**
 * States, applied over a level as translucent scrims so they composite over any surface and inherit its wash.
 *
 * Shares of {@link STATE_TONE}, except `selected`, which is a share of `secondary`.
 */
export const STATE = {
  hover: { light: 0.06, dark: 0.07 },
  /** What the editor has open. Its own value, so a hovered row and an open row are never drawn the same. */
  current: { light: 0.1, dark: 0.12 },
  /** The keyboard cursor, the top of the neutral ladder rather than a hue of its own. */
  selected: { light: 0.18, dark: 0.2 },
  /** A recess holding input or a nested listing. Always bordered: on the dark content plane fill alone is ~1.05:1. */
  well: { light: 0.08, dark: 0.3 },
  /**
   * MUI's own `selectedOpacity`, which several components apply to an *accent* rather than to a neutral.
   *
   * A separate number from `selected` because the jobs differ: `selected` tints a neutral hard enough to read as a
   * cursor, while this tints `primary` or `text.primary`, where the same share would shout. MUI's stock 0.08 is too
   * faint to see on a light plane.
   */
  accentSelected: { light: 0.12, dark: 0.16 },
  disabledOpacity: 0.38,
} as const;

// Neutral by rule: every surface, divider and text value has equal channels, so all chrome hue comes from the wash.
// A warm wash over a cool neutral cancels, and what survives is the grey that made light mode read as muddy.
export const TEXT = {
  primary: { light: "#1a1a1a", dark: "#d9d9d9" },
  secondary: { light: "#4a4a4a", dark: "#c4c4c4" },
} as const;

/**
 * Per-group wayfinding hues for the launcher, hand-authored because a categorical palette needs hue distinctness
 * that a lightness formula does not give. They tint icons rather than text, so they answer to the 3:1 tier.
 */
export const CATALOG_ACCENT = {
  archives: { light: "#1c982d", dark: "#5d9f4b" },
  configs: { light: "#5343c7", dark: "#a692ff" },
  dialogs: { light: "#b22747", dark: "#f87887" },
  environment: { light: "#986a13", dark: "#e9bd62" },
  gamedata: { light: "#20733d", dark: "#43d37a" },
  gameplay: { light: "#b24422", dark: "#ff875b" },
  level: { light: "#8a5a3b", dark: "#d8a77d" },
  materials: { light: "#77634c", dark: "#cfb18f" },
  particles: { light: "#a53679", dark: "#f07ec3" },
  scripts: { light: "#08778a", dark: "#2bd0df" },
  shaders: { light: "#6548a3", dark: "#b99aea" },
  sounds: { light: "#197f78", dark: "#62cfc6" },
  spawns: { light: "#59730c", dark: "#a4d83b" },
  sprites: { light: "#8934c4", dark: "#d087ff" },
  textures: { light: "#896400", dark: "#f3c53d" },
  translations: { light: "#007b64", dark: "#30d6af" },
  visuals: { light: "#006faa", dark: "#39b8ff" },
} as const;

/** Shared content-state measurements. Spacing values use theme units. */
export const CONTENT_STATE = {
  padding: 3,
  gap: 1,
  iconSize: 40,
  descriptionMaxWidth: 440,
} as const;

export const DIVIDER = {
  light: "#d2d2d2",
  dark: "#434343",
} as const;

/**
 * The title band's own sheen.
 */
export const HEADER_GLOSS = {
  highlight: { light: 0.6, dark: 0.01 },
  shade: { light: 0.05, dark: 0.1 },
  /** Where the highlight has fully faded, leaving the band its own colour before the shade begins. */
  fadeAt: "60%",
} as const;

/** A floating surface's own edge. It carries the separation in dark, where a shadow on near-black cannot. */
export const OVERLAY_BORDER = {
  light: "#e6e6e6",
  dark: "#3a3a3a",
} as const;

/** The only two shadows, spread by `creation.ts` across the twenty-five slots MUI's theme requires. */
export const SHADOW = {
  raised: {
    light: "0 1px 2px rgba(0, 0, 0, 0.06), 0 6px 16px rgba(0, 0, 0, 0.10)",
    dark: "0 1px 2px rgba(0, 0, 0, 0.13)",
  },
  overlay: {
    light: "0 1px 3px rgba(0, 0, 0, 0.05), 0 8px 24px rgba(0, 0, 0, 0.11)",
    dark: "0 1px 3px rgba(0, 0, 0, 0.34), 0 8px 24px rgba(0, 0, 0, 0.44)",
  },
} as const;
