/** What the feature sliders within a surface are measured by, and reset to. */
export enum ERenderSliderReferenceSource {
  /** The settings every viewport draws with, which a view's popover resets to. */
  SETTINGS = "settings",
  /** The preset the settings follow, which the settings dialog resets to. */
  PRESET = "preset",
}
