import { ReactElement, ReactNode } from "react";

interface ICaptionGlyphProps {
  children: ReactNode;
}

/**
 * Windows draws its caption glyphs on a 10x10 box with a hairline stroke, so these do too.
 *
 * Drawn rather than taken from an icon set: the icon fonts round their corners and thicken their
 * strokes, which reads as an application's own button instead of the window's.
 */
export function CaptionGlyph({ children }: ICaptionGlyphProps): ReactElement {
  return (
    <svg viewBox={"0 0 10 10"} aria-hidden={true} className={"size-2.5 fill-none stroke-current stroke-1"}>
      {children}
    </svg>
  );
}
