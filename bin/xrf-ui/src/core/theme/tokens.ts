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

/** Fixed color fields shared by launcher, picker, and planned screens. */
export const APPLICATION_BACKGROUND = {
  radiusX: 800,
  radiusY: 600,
  primary: { x: 24, y: 640, opacity: { light: "8%", dark: "2%" } },
  secondary: { x: 24, y: 16, opacity: { light: "7%", dark: "2%" } },
} as const;

export const LAYOUT = {
  railWidth: 44,
  /**
   * Every control in either stripe.
   *
   * Sized to sit inside `toolbarHeight` with room to breathe, because Home and Notifications share
   * that band with the toolbar and have to line up with its title.
   */
  railButtonSize: 32,
  /** Dense `MuiToolbar` and `MuiTab` still measure themselves against this; the window caption does not. */
  toolbarHeight: 40,
  statusBarHeight: 24,
  /**
   * The window's only top band: caption and the active application's toolbar in one row.
   */
  titleBarHeight: 36,
  windowControlWidth: 36,
  /**
   * A slider hosted in a toolbar popover.
   */
  toolbarSliderWidth: 200,
  /** The motion picker, wide enough for a name like `norm_walk_fwd_1` without truncating it. */
  motionPickerWidth: 260,
  /** Frame counter beside the motion slider, sized so a four digit frame of a four digit total does not reflow it. */
  motionCounterWidth: 118,
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
  primary: {
    main: { light: "#795108", dark: "#ffb51a" },
    contrastText: { light: "#ffffff", dark: "#241b06" },
  },
  secondary: {
    main: { light: "#1b5e90", dark: "#60bcff" },
    contrastText: { light: "#ffffff", dark: "#062138" },
  },
} as const;

export const STATUS = {
  success: {
    main: { light: "#3d671e", dark: "#9ccc65" },
    contrastText: { light: "#ffffff", dark: "#18230e" },
  },
  warning: {
    main: { light: "#8f490c", dark: "#ffa45b" },
    contrastText: { light: "#ffffff", dark: "#291807" },
  },
  error: {
    main: { light: "#a33028", dark: "#ff958a" },
    contrastText: { light: "#ffffff", dark: "#310c09" },
  },
} as const;

export const SURFACE = {
  default: { light: "#eef1f5", dark: "#171717" },
  paper: { light: "#f8fafc", dark: "#242424" },
  raised: { light: "#ffffff", dark: "#303030" },
  input: { light: "#ffffff", dark: "#1d1d1d" },
} as const;

export const TEXT = {
  primary: { light: "rgba(0, 0, 0, 0.87)", dark: "#d9d9d9" },
  secondary: { light: "#4f5c6d", dark: "#b7b7b7" },
} as const;

/** Shared content-state measurements. Spacing values use theme units. */
export const CONTENT_STATE = {
  padding: 3,
  gap: 1,
  iconSize: 40,
  descriptionMaxWidth: 440,
} as const;

export const DIVIDER = {
  light: "#d0d7e0",
  dark: "#434343",
} as const;
