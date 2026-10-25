import { ReactElement } from "react";

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
