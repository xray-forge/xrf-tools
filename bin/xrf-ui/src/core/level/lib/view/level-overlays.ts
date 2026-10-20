import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";

/** One overlay laid over the level, switched from the toolbar's Overlays popover and the level viewer's menu. */
export interface ILevelOverlay {
  /** Kebab case, for the ids of what switches it. */
  readonly id: string;
  readonly option: keyof Pick<
    ILevelViewOptions,
    "isGridVisible" | "isAxesVisible" | "isSunMarked" | "isStatsVisible" | "isMetered" | "isCalibrationVisible"
  >;
  readonly label: string;
  readonly description: string;
}

/** The overlays, in the order every surface lists them. */
export const LEVEL_OVERLAYS: ReadonlyArray<ILevelOverlay> = [
  {
    id: "grid",
    option: "isGridVisible",
    label: "Grid",
    description: "Draws the ground plane in round cells and the extent the level claims",
  },
  {
    id: "axes",
    option: "isAxesVisible",
    label: "Axes",
    description: "Marks the level's origin and which way +x and +z point",
  },
  {
    id: "sun",
    option: "isSunMarked",
    label: "Sun",
    description: "Marks where the level's directional light comes from with a dot in the sky",
  },
  {
    id: "readouts",
    option: "isStatsVisible",
    label: "Readouts",
    description: "Shows what each frame costs and where the camera stands",
  },
  {
    id: "meter",
    option: "isMetered",
    label: "Spot meter",
    description: "Reads the light at the centre, in stops about middle grey, into the readouts",
  },
  {
    id: "calibration",
    option: "isCalibrationVisible",
    label: "Calibration props",
    description: "Stands a grey and a white sphere, a mirror ball and a colour chart ahead of the camera",
  },
];
