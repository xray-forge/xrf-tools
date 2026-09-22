import { EventType } from "@wirestate/core";

/** Everything the settings dialog can change, by the name the change is announced under. */
export enum ESetting {
  CATALOG_VIEW = "catalogView",
  DEV_MODE = "devMode",
  FRAME_RATE_LIMIT = "frameRateLimit",
  OFFSCREEN_RENDER = "offscreenRender",
}

/** Announces that a setting now says something else. */
export const SETTINGS_CHANGED_EVENT: EventType = Symbol("@/settings/changed");

/**
 * Which setting changed.
 */
export interface ISettingsChangedPayload<T> {
  setting: ESetting;
  value: T;
}
