/**
 * Every local storage key this application writes.
 *
 * One module rather than a literal spelled where each key is used, because two surfaces have to agree about the whole
 * space rather than about one key at a time: the Settings storage section groups what is stored, and a key classified
 * by a second list that has to be kept in step is how four keys in three naming styles got here before. Kept together
 * for the same reason - split by namespace, the inventory would answer "which file declares the volume key?" instead
 * of reading at a glance.
 */

import { buildStorageKey, EStorageNamespace } from "@/core/storage/namespace";

/** Colour scheme the application opens in - `light`, `dark` or `system`. */
export const THEME_STORAGE_KEY: string = buildStorageKey(EStorageNamespace.PREFERENCE, "theme");
/** Whether dev traces and captured runtime errors are surfaced. */
export const DEV_MODE_STORAGE_KEY: string = buildStorageKey(EStorageNamespace.PREFERENCE, "dev-mode");
/** How the root catalog draws its tools. */
export const CATALOG_VIEW_STORAGE_KEY: string = buildStorageKey(EStorageNamespace.PREFERENCE, "catalog-view");
/** Whether IPC payloads are weighed. */
export const IPC_PROFILING_STORAGE_KEY: string = buildStorageKey(EStorageNamespace.PREFERENCE, "ipc-profiling");
/** How loud playback is. */
export const MEDIA_VOLUME_STORAGE_KEY: string = buildStorageKey(EStorageNamespace.PREFERENCE, "media-volume");

/**
 * @param application - Application the field belongs to.
 * @param id - Field name within that application.
 * @returns The storage key.
 */
export function getFieldValueStorageKey(application: string, id: string): string {
  return buildStorageKey(EStorageNamespace.FORM, application, id);
}

/**
 * @param application - Application the field belongs to.
 * @param id - Field name within that application.
 * @returns The storage key.
 */
export function getFieldRecentsStorageKey(application: string, id: string): string {
  return buildStorageKey(EStorageNamespace.FORM_RECENTS, application, id);
}

/**
 * @param side - Side the panel is docked to.
 * @returns The storage key.
 */
export function getPanelWidthStorageKey(side: string): string {
  return buildStorageKey(EStorageNamespace.PANELS, side, "width");
}

/**
 * @param side - Side the panel is docked to.
 * @param scope - What the selection applies to, so two surfaces do not share one answer.
 * @returns The storage key.
 */
export function getPanelSelectionStorageKey(side: string, scope: string): string {
  return buildStorageKey(EStorageNamespace.PANELS, side, scope);
}
