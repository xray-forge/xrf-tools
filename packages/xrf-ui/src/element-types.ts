import { SxProps, Theme } from "@mui/material/styles";

/** Identity for a component's rendered surface. */
export interface BaseComponentProps {
  ["data-testid"]?: string;
  id?: string;
  className?: string;
}

/** Opt-in style customization for components that forward caller styles. */
export interface StyledComponentProps extends BaseComponentProps {
  /** Applied after default styles, supporting objects, theme callbacks, and ordered arrays. */
  sx?: SxProps<Theme>;
}
