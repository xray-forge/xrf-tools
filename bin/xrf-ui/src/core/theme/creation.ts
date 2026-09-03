import { createTheme, PaletteOptions, Theme } from "@mui/material/styles";
// Type-only, side-effect import: it pulls in `@mui/x-data-grid`'s module augmentation,
// which registers the `MuiDataGrid` slot on MUI's `Components` type.
import type {} from "@mui/x-data-grid/themeAugmentation";

import { ACCENT, CONTROL, DIVIDER, LAYOUT, MONOSPACE, RADIUS, STATUS, SURFACE, TEXT } from "@/core/theme/tokens";

type ColorScheme = "light" | "dark";

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
    background: { default: SURFACE.default[scheme], paper: SURFACE.paper[scheme] },
    text: { primary: TEXT.primary[scheme], secondary: TEXT.secondary[scheme] },
    divider: DIVIDER[scheme],
  };
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
          root: ({ theme }) => ({
            backgroundColor: (theme.vars ?? theme).palette.background.paper,
            borderBottom: `1px solid ${(theme.vars ?? theme).palette.divider}`,
          }),
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
          root: { paddingTop: 4, paddingBottom: 4 },
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
      },
      MuiCard: {
        defaultProps: { variant: "outlined" },
        styleOverrides: {
          root: ({ theme }) => ({
            borderRadius: RADIUS.lg,
            borderColor: (theme.vars ?? theme).palette.divider,
          }),
        },
      },
      MuiButton: {
        defaultProps: { disableElevation: true },
      },
      MuiDialog: {
        defaultProps: { closeAfterTransition: false },
      },
      MuiTextField: {
        defaultProps: { size: "small" },
      },
      MuiOutlinedInput: {
        styleOverrides: {
          root: ({ theme }) => ({
            borderRadius: RADIUS.sm,
            backgroundColor: (theme.vars ?? theme).palette.action.hover,
            "& .MuiOutlinedInput-notchedOutline": {
              borderColor: (theme.vars ?? theme).palette.divider,
            },
            // One height for every small control, whatever type it renders.
            "&.MuiInputBase-sizeSmall:not(.MuiInputBase-multiline) .MuiOutlinedInput-input": {
              height: CONTROL.smallHeight - CONTROL.smallInputPaddingY * 2,
              minHeight: CONTROL.smallHeight - CONTROL.smallInputPaddingY * 2,
            },
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
            // Opt-in per column: identifiers, paths and file positions are compared by eye and read
            // badly in proportional type.
            "& .monospace": MONOSPACE,
          },
        },
      },
    },
  });
}
