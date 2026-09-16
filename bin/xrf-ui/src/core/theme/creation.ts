import { cardActionAreaClasses, inputBaseClasses, listItemButtonClasses, outlinedInputClasses } from "@mui/material";
import { createTheme, PaletteOptions, Theme } from "@mui/material/styles";

// Type-only, side-effect import: it pulls in `@mui/x-data-grid`'s module augmentation,
// which registers the `MuiDataGrid` slot on MUI's `Components` type.
import type {} from "@mui/material/themeCssVarsAugmentation";
import type {} from "@mui/x-data-grid/themeAugmentation";

import { getHeaderGlossImage, getWashImage } from "./surface";
import {
  ACCENT,
  CONTROL,
  DIALOG,
  DIVIDER,
  LAYOUT,
  MONOSPACE,
  OVERLAY_BORDER,
  RADIUS,
  RECESS_TONE,
  SHADOW,
  STATE,
  STATE_TONE,
  STATUS,
  SURFACE,
  TEXT,
  toSharePercent,
} from "./tokens";

declare module "@mui/material/styles" {
  interface TypeBackground {
    /** The window's furniture: title bar, status bar, rails, and the docked panels welded to them. */
    frame: string;
  }

  interface TypeAction {
    /** What the editor has open, as opposed to where the keyboard stands. */
    current: string;
    /** Authored rather than derived, see {@link toChannel}. Optional: MUI derives it when it is absent. */
    selectedChannel?: string;
  }
}

type ColorScheme = "light" | "dark";

/** Shadows flip per scheme, which MUI's flat `shadows` tuple cannot express, so the tuple holds variables. */
const SHADOW_RAISED: string = "var(--xrf-shadow-raised)";
const SHADOW_OVERLAY: string = "var(--xrf-shadow-overlay)";

/** A neutral state scrim, translucent so it composites over whichever level hosts it. */
function toScrim(scheme: ColorScheme, share: number): string {
  return `color-mix(in srgb, ${STATE_TONE[scheme]} ${toSharePercent(share)}, transparent)`;
}

/**
 * Splits an opaque `#rrggbb` tone into the channels a MUI `<key>Channel` token carries.
 *
 * @param tone - Opaque colour in `#rrggbb` form.
 * @returns The tone's red, green, and blue channels, space separated.
 */
function toChannel(tone: string): string {
  const value: number = Number.parseInt(tone.slice(1), 16);

  return `${(value >> 16) & 0xff} ${(value >> 8) & 0xff} ${value & 0xff}`;
}

/**
 * Maps the design tokens onto a MUI palette for one color scheme.
 *
 * @param scheme - Color scheme whose token values populate the palette.
 * @returns Palette options for the requested color scheme.
 */
function createColorSchemePalette(scheme: ColorScheme): PaletteOptions {
  return {
    primary: { main: ACCENT.primary.main[scheme], contrastText: ACCENT.primary.contrastText[scheme] },
    secondary: { main: ACCENT.secondary.main[scheme], contrastText: ACCENT.secondary.contrastText[scheme] },
    success: { main: STATUS.success.main[scheme] },
    warning: { main: STATUS.warning.main[scheme] },
    error: { main: STATUS.error.main[scheme] },
    background: {
      default: SURFACE.content[scheme],
      paper: SURFACE.overlay[scheme],
      frame: SURFACE.frame[scheme],
    },
    text: { primary: TEXT.primary[scheme], secondary: TEXT.secondary[scheme] },
    divider: DIVIDER[scheme],
    action: {
      hover: toScrim(scheme, STATE.hover[scheme]),
      hoverOpacity: STATE.hover[scheme],
      current: toScrim(scheme, STATE.current[scheme]),
      focus: toScrim(scheme, STATE.current[scheme]),
      focusOpacity: STATE.current[scheme],
      activatedOpacity: STATE.current[scheme],
      selected: toScrim(scheme, STATE.selected[scheme]),
      selectedOpacity: STATE.accentSelected[scheme],
      selectedChannel: toChannel(STATE_TONE[scheme]),
      disabledOpacity: STATE.disabledOpacity,
    },
  };
}

/** Twenty-five slots, three authored values: nothing between `raised` and `overlay` is a distinction we draw. */
function createShadows(): Theme["shadows"] {
  return [
    "none",
    ...new Array<string>(4).fill(SHADOW_RAISED),
    ...new Array<string>(20).fill(SHADOW_OVERLAY),
  ] as Theme["shadows"];
}

/** A well's own fill. Kept out of the palette: it is a recess on a level, not a level of its own. */
function toWellFill(scheme: ColorScheme): string {
  return `color-mix(in srgb, ${RECESS_TONE} ${toSharePercent(STATE.well[scheme])}, transparent)`;
}

export function createApplicationTheme(): Theme {
  return createTheme({
    cssVariables: {
      colorSchemeSelector: "data-color-scheme",
    },
    defaultColorScheme: "dark",
    shape: {
      borderRadius: RADIUS.md,
    },
    shadows: createShadows(),
    typography: {
      // Segoe first: it is the strongest native signal on windows and covers cyrillic on its own.
      // Roboto stays as a bundled fallback for platforms without segoe.
      fontFamily: ["'Segoe UI Variable Text'", "'Segoe UI'", "'Roboto'", "system-ui", "sans-serif"].join(", "),
      fontSize: 13,
      h5: { fontWeight: 600 },
      h6: { fontWeight: 600, fontSize: "0.9375rem" },
      button: {
        textTransform: "none",
        fontWeight: 500,
      },
    },
    colorSchemes: {
      light: { palette: createColorSchemePalette("light") },
      dark: { palette: createColorSchemePalette("dark") },
    },
    components: {
      // Thin, unobtrusive scrollbars. The default chromium ones are wide enough to read as a web page.
      MuiCssBaseline: {
        styleOverrides: (theme) => ({
          // Written as explicit selectors rather than through `applyStyles`: the scheme attribute sits on `html`
          // itself, so a descendant selector would never match it. `dark` is the default scheme, hence bare `:root`.
          ":root": {
            "--xrf-shadow-raised": SHADOW.raised.dark,
            "--xrf-shadow-overlay": SHADOW.overlay.dark,
          },
          '[data-color-scheme="light"]': {
            "--xrf-shadow-raised": SHADOW.raised.light,
            "--xrf-shadow-overlay": SHADOW.overlay.light,
          },
          // Outrank MUI typography defaults regardless of style injection order.
          ".monospace.monospace": MONOSPACE,
          "*::-webkit-scrollbar": { width: 10, height: 10 },
          "*::-webkit-scrollbar-track": { backgroundColor: "transparent" },
          "*::-webkit-scrollbar-thumb": {
            backgroundColor: (theme.vars ?? theme).palette.divider,
            borderRadius: RADIUS.sm,
          },
          "*::-webkit-scrollbar-thumb:hover": {
            backgroundColor: (theme.vars ?? theme).palette.text.secondary,
          },
        }),
      },
      // Flat surfaces: MUI's dark elevation overlay tints `paper`.
      MuiPaper: {
        styleOverrides: {
          root: { backgroundImage: "none" },
        },
      },
      // `caption`, `button` and `overline` render as a span, and MUI's own `noWrap` rules do nothing to an inline
      // box: such a line ran past its container and was clipped by an ancestor instead of ending in an ellipsis.
      MuiTypography: {
        styleOverrides: {
          noWrap: { display: "block" },
        },
      },
      // Neutral command bar instead of material's filled accent bar.
      MuiAppBar: {
        defaultProps: { color: "default", elevation: 0 },
        styleOverrides: {
          // No rule beneath it: the caption, the rails and the status bar are one continuous frame, and a divider
          // between two pieces of the same level reads as a seam that is not there.
          root: ({ theme }) => ({ backgroundColor: (theme.vars ?? theme).palette.background.frame }),
        },
      },
      MuiToolbar: {
        defaultProps: { variant: "dense" },
        styleOverrides: {
          dense: { minHeight: LAYOUT.toolbarHeight },
          // Tight enough that the leading control sits close to the window edge, like a desktop
          // command bar rather than a web header.
          gutters: ({ theme }) => ({
            paddingLeft: 4,
            paddingRight: 4,
            [theme.breakpoints.up("sm")]: { paddingLeft: 4, paddingRight: 4 },
          }),
        },
      },
      MuiListItemButton: {
        styleOverrides: {
          // MUI tints `primary.main` here by default, which made a chosen list row the only selection in the
          // application that was not the shared `selected` tint.
          root: ({ theme }) => ({
            paddingTop: 4,
            paddingBottom: 4,
            [`&.${listItemButtonClasses.selected}`]: {
              backgroundColor: (theme.vars ?? theme).palette.action.selected,
              [`&:hover, &.${listItemButtonClasses.focusVisible}`]: {
                backgroundColor: (theme.vars ?? theme).palette.action.selected,
              },
            },
          }),
        },
      },
      // An accordion is a `Paper`, so it inherited the overlay level while sitting inline on a page. It is a
      // recess in whatever hosts it, not something floating above one.
      MuiAccordion: {
        defaultProps: { elevation: 0, disableGutters: true },
        styleOverrides: {
          root: ({ theme }) => ({
            backgroundColor: toWellFill("light"),
            border: `1px solid ${(theme.vars ?? theme).palette.divider}`,
            borderRadius: RADIUS.md,
            "&::before": { display: "none" },
            ...theme.applyStyles("dark", { backgroundColor: toWellFill("dark") }),
          }),
        },
      },
      MuiListItemIcon: {
        styleOverrides: {
          root: { minWidth: 32 },
        },
      },
      MuiListItemText: {
        styleOverrides: {
          primary: { fontSize: "0.8125rem" },
          secondary: { fontSize: "0.75rem" },
        },
      },
      MuiTabs: {
        styleOverrides: {
          root: { minHeight: LAYOUT.toolbarHeight },
        },
      },
      MuiTab: {
        styleOverrides: {
          root: { minHeight: LAYOUT.toolbarHeight, textTransform: "none", fontSize: "0.8125rem" },
        },
      },
      MuiIconButton: {
        defaultProps: { size: "small" },
      },
      MuiTooltip: {
        defaultProps: { enterDelay: 400 },
        styleOverrides: {
          tooltip: ({ theme }) => ({
            backgroundColor: (theme.vars ?? theme).palette.background.paper,
            color: (theme.vars ?? theme).palette.text.primary,
            border: `1px solid ${OVERLAY_BORDER.light}`,
            boxShadow: SHADOW_OVERLAY,
            fontSize: "0.75rem",
            ...theme.applyStyles("dark", { border: `1px solid ${OVERLAY_BORDER.dark}` }),
          }),
        },
      },
      // Glass is permitted here because both consumers sit on the application background, so the backdrop is a
      // known range rather than arbitrary content. The inset bevels are card-only and must not become general.
      MuiCard: {
        defaultProps: { variant: "outlined" },
        styleOverrides: {
          root: ({ theme }) => ({
            borderRadius: LAYOUT.surfaceRadius,
            borderColor: `color-mix(in srgb, ${(theme.vars ?? theme).palette.text.primary} 12%, transparent)`,
            backgroundColor: (theme.vars ?? theme).palette.background.frame,
            "--xrf-card-opacity": "75%",
            "--xrf-card-disabled-opacity": "35%",
            "--xrf-card-hover-opacity": "25%",
            "--xrf-card-edge": "45%",
            "@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))": {
              backgroundColor: `color-mix(in srgb, ${(theme.vars ?? theme).palette.background.frame} var(--xrf-card-opacity), transparent)`,
              backdropFilter: "blur(18px) saturate(140%)",
              WebkitBackdropFilter: "blur(18px) saturate(140%)",
              boxShadow: [
                `inset 1px 1px 0 color-mix(in srgb, ${theme.palette.common.white} var(--xrf-card-edge), transparent)`,
                `inset -1px -1px 0 color-mix(in srgb, ${theme.palette.common.white} 5%, transparent)`,
                SHADOW_RAISED,
              ].join(", "),
            },
            ...theme.applyStyles("dark", {
              "--xrf-card-edge": "10%",
            }),
            transition: "background-color 140ms ease, border-color 140ms ease",
            [`&:has(> .${cardActionAreaClasses.root}):hover`]: {
              "--xrf-card-opacity": "var(--xrf-card-hover-opacity)",
              borderColor: (theme.vars ?? theme).palette.primary.main,
            },
          }),
        },
      },
      MuiButton: {
        defaultProps: { disableElevation: true },
      },
      MuiDialog: {
        defaultProps: { closeAfterTransition: false },
        styleOverrides: {
          paper: ({ theme }) => ({
            border: `1px solid ${OVERLAY_BORDER.light}`,
            ...theme.applyStyles("dark", { border: `1px solid ${OVERLAY_BORDER.dark}` }),
          }),
        },
      },
      // In light both hold `#ffffff`, so the border and the shadow are the whole of what says "above".
      MuiPopover: {
        styleOverrides: {
          paper: ({ theme }) => ({
            border: `1px solid ${OVERLAY_BORDER.light}`,
            boxShadow: SHADOW_OVERLAY,
            ...theme.applyStyles("dark", { border: `1px solid ${OVERLAY_BORDER.dark}` }),
          }),
        },
      },
      MuiMenu: {
        styleOverrides: {
          paper: ({ theme }) => ({
            border: `1px solid ${OVERLAY_BORDER.light}`,
            boxShadow: SHADOW_OVERLAY,
            ...theme.applyStyles("dark", { border: `1px solid ${OVERLAY_BORDER.dark}` }),
          }),
        },
      },
      MuiDialogTitle: {
        styleOverrides: {
          root: ({ theme }) => ({
            boxSizing: "border-box",
            minHeight: DIALOG.headerMinHeight,
            padding: theme.spacing(DIALOG.headerPaddingY, DIALOG.paddingX),
            backgroundColor: (theme.vars ?? theme).palette.background.frame,
            backgroundImage: getHeaderGlossImage("light"),
            ...theme.applyStyles("dark", { backgroundImage: getHeaderGlossImage("dark") }),
          }),
        },
      },
      // The body reads like a file open in an explorer, so it takes the reading plane while the title and the
      // actions keep the overlay they float on. In light the two levels hold one value and the dividers separate them.
      MuiDialogContent: {
        defaultProps: { dividers: true },
        styleOverrides: {
          root: ({ theme }) => ({
            backgroundColor: (theme.vars ?? theme).palette.background.default,
            backgroundAttachment: "fixed",
            backgroundImage: getWashImage("light"),
            padding: theme.spacing(DIALOG.contentPaddingY, DIALOG.paddingX),
            ...theme.applyStyles("dark", { backgroundImage: getWashImage("dark") }),
          }),
        },
      },
      MuiDialogActions: {
        defaultProps: { disableSpacing: true },
        styleOverrides: {
          root: ({ theme }) => ({
            gap: theme.spacing(DIALOG.gap),
            padding: theme.spacing(DIALOG.actionsPaddingY, DIALOG.paddingX),
            backgroundColor: (theme.vars ?? theme).palette.background.frame,
            backgroundImage: getHeaderGlossImage("light"),
            ...theme.applyStyles("dark", { backgroundImage: getHeaderGlossImage("dark") }),
          }),
        },
      },
      MuiTextField: {
        defaultProps: { size: "small" },
      },
      // A field is a well: translucent so the host's wash shows through, and bordered because on the dark content
      // plane there is no room left beneath it for fill to say anything.
      MuiOutlinedInput: {
        styleOverrides: {
          root: ({ theme }) => ({
            borderRadius: RADIUS.sm,
            backgroundColor: toWellFill("light"),
            [`& .${outlinedInputClasses.notchedOutline}`]: {
              borderColor: (theme.vars ?? theme).palette.divider,
            },
            // One height for every small control, whatever type it renders.
            [`&.${inputBaseClasses.sizeSmall}:not(.${inputBaseClasses.multiline}) .${outlinedInputClasses.input}`]: {
              height: CONTROL.smallHeight - CONTROL.smallInputPaddingY * 2,
              minHeight: CONTROL.smallHeight - CONTROL.smallInputPaddingY * 2,
            },
            ...theme.applyStyles("dark", { backgroundColor: toWellFill("dark") }),
          }),
        },
      },
      MuiDataGrid: {
        defaultProps: {
          density: "compact",
          disableRowSelectionOnClick: true,
        },
        styleOverrides: {
          root: {
            border: "none",
          },
        },
      },
    },
  });
}
