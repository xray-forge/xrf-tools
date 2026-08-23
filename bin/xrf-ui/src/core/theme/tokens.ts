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
 * Explorer trees, shared by every surface that renders one.
 *
 * Sized against `LAYOUT.railButtonSize` rather than freely: a tree row sits in the same panels as the rail's controls,
 * and rows that disagree with them read as a different application.
 */
export const TREE = {
  /** Row height, dense enough that a mesh directory is scannable without becoming a hit-target problem. */
  rowHeight: 28,
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
} as const;

export const ACCENT = {
  primary: {
    main: { light: "#8a5e0c", dark: "#ffb51a" },
    contrastText: { light: "#ffffff", dark: "#241b06" },
  },
  secondary: {
    main: { light: "#1e6699", dark: "#60bcff" },
    contrastText: { light: "#ffffff", dark: "#062138" },
  },
} as const;

export const STATUS = {
  success: { main: { light: "#5a8f2e", dark: "#9ccc65" } }, // toxic green
  warning: { main: { light: "#c2641a", dark: "#f2933e" } }, // burnt orange
  error: { main: { light: "#b23b30", dark: "#e0564a" } }, // warm red
} as const;

export const SURFACE = {
  default: { light: "#eef0f2", dark: "#161719" },
  paper: { light: "#f7f8fa", dark: "#1e2023" },
} as const;

export const TEXT = {
  primary: { light: "rgba(0, 0, 0, 0.87)", dark: "#d7dadc" },
  secondary: { light: "rgba(0, 0, 0, 0.6)", dark: "#969b9f" },
} as const;

export const DIVIDER = {
  light: "rgba(20, 28, 35, 0.1)",
  dark: "rgba(220, 228, 235, 0.08)",
} as const;
