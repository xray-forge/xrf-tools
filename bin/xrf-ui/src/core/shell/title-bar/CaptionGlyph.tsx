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

export function MinimizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0 5h10"} />
    </CaptionGlyph>
  );
}

export function MaximizeGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <rect x={"0.5"} y={"0.5"} width={"9"} height={"9"} />
    </CaptionGlyph>
  );
}

/** Two offset squares, the same way the system draws a window that can be put back where it was. */
export function RestoreGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M2.5 2.5V0.5h7v7h-2"} />
      <rect x={"0.5"} y={"2.5"} width={"7"} height={"7"} />
    </CaptionGlyph>
  );
}

export function CloseGlyph(): ReactElement {
  return (
    <CaptionGlyph>
      <path d={"M0.5 0.5l9 9m0-9l-9 9"} />
    </CaptionGlyph>
  );
}

/**
 * Three rounded lines on a 16x16 box, as desktop applications draw the button that shows their menu: a stroke and a half,
 * since a hairline thins to grey once scaled, and set half a pixel low, where the caption text's capitals centre.
 */
export function MenuGlyph(): ReactElement {
  return (
    <svg
      viewBox={"0 0 16 16"}
      aria-hidden={true}
      className={"size-4 translate-y-[0.5px] fill-none stroke-current stroke-[1.5]"}
      strokeLinecap={"round"}
    >
      <path d={"M2 3.5h12M2 8h12M2 12.5h12"} />
    </svg>
  );
}
