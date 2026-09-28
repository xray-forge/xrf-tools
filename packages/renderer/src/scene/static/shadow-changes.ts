import { IShadowChange } from "#/scene/static/shadow-change";

/** What changed since a version: every change kept, or anywhere, the log no longer reaching back. */
export interface IShadowChanges {
  readonly isEverywhere: boolean;
  readonly changes: ReadonlyArray<IShadowChange>;
}
