/** A box of the page in CSS pixels, its edges as `getBoundingClientRect` gives them. */
export interface ICssBox {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** A length for each side of a box, in CSS pixels. */
export interface ICssEdges {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/** A radius for each corner of a box, in CSS pixels. */
export interface ICssCorners {
  topLeft: number;
  topRight: number;
  bottomRight: number;
  bottomLeft: number;
}

/** How far apart two edges may be, in CSS pixels, and still be one edge: layout rounds to fractions of a pixel. */
const EDGE_TOLERANCE: number = 0.5;

/** The computed `overflow` values that clip; anything else, `visible` or not computed at all, does not. */
const CLIPPING_OVERFLOWS: ReadonlySet<string> = new Set(["auto", "clip", "hidden", "scroll"]);

/**
 * @param style - An element's computed style.
 * @returns Its border widths.
 */
export function readBorderWidths(style: CSSStyleDeclaration): ICssEdges {
  return {
    bottom: readLength(style.borderBottomWidth),
    left: readLength(style.borderLeftWidth),
    right: readLength(style.borderRightWidth),
    top: readLength(style.borderTopWidth),
  };
}

/**
 * @param style - An element's computed style.
 * @returns Its border radii; a percentage or an elliptical radius reads as its first length.
 */
export function readBorderRadii(style: CSSStyleDeclaration): ICssCorners {
  return {
    bottomLeft: readLength(style.borderBottomLeftRadius),
    bottomRight: readLength(style.borderBottomRightRadius),
    topLeft: readLength(style.borderTopLeftRadius),
    topRight: readLength(style.borderTopRightRadius),
  };
}

/**
 * @param style - An element's computed style.
 * @returns Whether it clips what it holds to its padding box, as every `overflow` but `visible` does.
 */
export function isClipping(style: CSSStyleDeclaration): boolean {
  return CLIPPING_OVERFLOWS.has(style.overflowX) || CLIPPING_OVERFLOWS.has(style.overflowY);
}

/**
 * @param border - An element's border box.
 * @param borders - Its border widths.
 * @returns Its padding box: what it clips its content to.
 */
export function toPaddingBox(border: ICssBox, borders: ICssEdges): ICssBox {
  return {
    bottom: border.bottom - borders.bottom,
    left: border.left + borders.left,
    right: border.right - borders.right,
    top: border.top + borders.top,
  };
}

/**
 * @param radii - An element's border radii.
 * @param borders - Its border widths.
 * @returns Its padding box's radii: each border radius less the wider border meeting at that corner.
 */
export function toPaddingRadii(radii: ICssCorners, borders: ICssEdges): ICssCorners {
  return {
    bottomLeft: Math.max(0, radii.bottomLeft - Math.max(borders.bottom, borders.left)),
    bottomRight: Math.max(0, radii.bottomRight - Math.max(borders.bottom, borders.right)),
    topLeft: Math.max(0, radii.topLeft - Math.max(borders.top, borders.left)),
    topRight: Math.max(0, radii.topRight - Math.max(borders.top, borders.right)),
  };
}

/**
 * @returns The part of the first box inside the second.
 */
export function intersectBoxes(first: ICssBox, second: ICssBox): ICssBox {
  return {
    bottom: Math.min(first.bottom, second.bottom),
    left: Math.max(first.left, second.left),
    right: Math.min(first.right, second.right),
    top: Math.max(first.top, second.top),
  };
}

/**
 * @returns Whether the first box reaches every edge of the second.
 */
export function isCovering(first: ICssBox, second: ICssBox): boolean {
  return (
    first.left <= second.left + EDGE_TOLERANCE &&
    first.top <= second.top + EDGE_TOLERANCE &&
    first.right >= second.right - EDGE_TOLERANCE &&
    first.bottom >= second.bottom - EDGE_TOLERANCE
  );
}

/**
 * @param inner - A box inside the other.
 * @param outer - The box holding it.
 * @param radii - The outer box's corner radii.
 * @returns The outer radii at the corners the inner box reaches, nothing at the others.
 */
export function getSharedCorners(inner: ICssBox, outer: ICssBox, radii: ICssCorners): ICssCorners {
  const isLeft: boolean = isSameEdge(inner.left, outer.left);
  const isTop: boolean = isSameEdge(inner.top, outer.top);
  const isRight: boolean = isSameEdge(inner.right, outer.right);
  const isBottom: boolean = isSameEdge(inner.bottom, outer.bottom);

  return {
    bottomLeft: isBottom && isLeft ? radii.bottomLeft : 0,
    bottomRight: isBottom && isRight ? radii.bottomRight : 0,
    topLeft: isTop && isLeft ? radii.topLeft : 0,
    topRight: isTop && isRight ? radii.topRight : 0,
  };
}

/**
 * @returns Each corner's larger radius.
 */
export function maxCorners(first: ICssCorners, second: ICssCorners): ICssCorners {
  return {
    bottomLeft: Math.max(first.bottomLeft, second.bottomLeft),
    bottomRight: Math.max(first.bottomRight, second.bottomRight),
    topLeft: Math.max(first.topLeft, second.topLeft),
    topRight: Math.max(first.topRight, second.topRight),
  };
}

function isSameEdge(first: number, second: number): boolean {
  return Math.abs(first - second) <= EDGE_TOLERANCE;
}

function readLength(length: string): number {
  return Number.parseFloat(length) || 0;
}
