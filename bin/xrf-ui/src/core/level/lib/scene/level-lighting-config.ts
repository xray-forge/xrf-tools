/** Where the sun sits for a level that has reported no extent yet, in metres. */
export const DEFAULT_SUN_DISTANCE: number = 500;

/** Times the level's own radius the sun is put at, so it clears the geometry it lights. */
export const SUN_DISTANCE_MARGIN: number = 2;

/** Metres ahead of the camera the sun marker is drawn, well inside the far plane of any level. */
export const SUN_MARKER_DISTANCE: number = 300;

/** Radius of the sun marker, which at its distance comes to about a degree and a half across. */
export const SUN_MARKER_RADIUS: number = 8;

/** Segments the marker's sphere is built from, enough to read as a disc at the size it is drawn. */
export const SUN_MARKER_SEGMENTS: number = 16;
