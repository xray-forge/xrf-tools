import { Nullable } from "@xrf/types";

/** What an observer reads of a render object: the bundle group recording it, if any, and the camera it draws with. */
export interface IObservedObject {
  bundle: Nullable<{ version: number }>;
  camera?: unknown;
  /** Its render context, which three gives the camera of every render call. */
  context?: { camera?: unknown };
}
