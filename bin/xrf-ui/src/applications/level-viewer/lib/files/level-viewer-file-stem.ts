import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";

/**
 * @param level - The level a file is of, as a snapshot names it, or null for none.
 * @param at - When it is taken.
 * @returns A file name without its extension naming the level and the moment, which sorts by time within a level.
 */
export function toLevelViewerFileStem(level: Nullable<string>, at: Date): string {
  const date: string = `${at.getFullYear()}${pad(at.getMonth() + 1)}${pad(at.getDate())}`;
  const time: string = `${pad(at.getHours())}${pad(at.getMinutes())}${pad(at.getSeconds())}`;

  return `${level ? getPathName(level) : "view"}-${date}-${time}`;
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}
