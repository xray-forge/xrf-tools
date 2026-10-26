import { ABSENT_VALUE } from "@xrf/format";

import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

/** The parts of a snapshot that are not renderer settings, by their key, as the dialog heads them. */
const GROUP_LABELS: Readonly<Record<string, string>> = {
  camera: "Camera",
  look: "Look",
  options: "Toolbar",
  output: "Output",
  settings: "Renderer",
  weather: "Weather",
};

/**
 * @param group - A part of the view as entries name it: `camera`, `settings.ambientOcclusion`.
 * @returns Its heading: a fixed name, or a renderer group's key in words.
 */
export function toLevelViewerImportGroupLabel(group: string): string {
  const words: string = group
    .replace(/^settings\./, "")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .toLowerCase();

  return GROUP_LABELS[group] ?? `${words.charAt(0).toUpperCase()}${words.slice(1)}`;
}

/**
 * @param key - The value's key, which says how it reads: the weather's clock as a time of day.
 * @param value - The value, undefined for none.
 * @returns The value as the dialog shows it.
 */
export function formatLevelViewerImportValue(key: string, value: unknown): string {
  if (value === undefined) {
    return ABSENT_VALUE;
  }

  if (key === "weather.time" && typeof value === "number") {
    return formatLevelWeatherTime(value);
  }

  if (typeof value === "boolean") {
    return value ? "on" : "off";
  }

  if (typeof value === "number") {
    return String(Math.round(value * 1000) / 1000);
  }

  if (Array.isArray(value)) {
    return value.map((item: unknown) => formatLevelViewerImportValue(key, item)).join(", ");
  }

  return value === null ? "none" : String(value);
}
