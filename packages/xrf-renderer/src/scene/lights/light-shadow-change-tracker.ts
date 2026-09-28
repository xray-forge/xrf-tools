import { Box3 } from "three/webgpu";

import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { LightShadowSlots } from "#/scene/lights/light-shadow-slots";
import { IShadowChange } from "#/scene/static/shadow-change";
import { IShadowChanges } from "#/scene/static/shadow-changes";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { isBoxInPlanes } from "#/visibility/plane-tests";

/**
 * Brings the lights' faces up to what the shadow views draw: stale where a change reached them, and moving as fast as
 * the fastest caster standing in them.
 */
export class LightShadowChangeTracker {
  private readonly changes: StaticShadowChanges;
  /** The change log's version the faces were last brought up to. */
  private version: number;
  /** The entries a change of something that sways or moves reached, whose motion is found once. */
  private readonly moved: Set<ILightShadowEntry> = new Set();

  /**
   * @param changes - Where what the shadow views draw changed, and what sways or moves.
   */
  public constructor(changes: StaticShadowChanges) {
    this.changes = changes;
    this.version = changes.version;
  }

  /**
   * Marks stale every face a change since the last update reached, and finds again what moves in the ones it could
   * have moved.
   *
   * @param slots - Every light's faces.
   */
  public update(slots: LightShadowSlots): void {
    const since: IShadowChanges = this.changes.since(this.version);

    this.version = this.changes.version;

    if (since.isEverywhere) {
      slots.forEachEntry((entry: ILightShadowEntry) => {
        entry.faces.forEach((face: ILightShadowFace) => (face.isStale = true));
        this.findMotion(entry);
      });

      return;
    }

    // Found once an entry, whatever reached it: a level arriving logs thousands of trees in one frame, and each
    // finding tests every caster that sways.
    since.changes.forEach((change: IShadowChange) => this.markChanged(slots, change.box as Box3, change.isAnimated));
    this.moved.forEach((entry: ILightShadowEntry) => this.findMotion(entry));
    this.moved.clear();
  }

  /**
   * @param entry - A light's faces, whose motion is found from the casters standing in each.
   */
  public findMotion(entry: ILightShadowEntry): void {
    entry.faces.forEach((face: ILightShadowFace) => (face.motion = this.changes.getMotion(face.planes, entry.sphere)));
  }

  private markChanged(slots: LightShadowSlots, box: Box3, isAnimated: boolean): void {
    slots.forEachEntry((entry: ILightShadowEntry) => {
      if (!entry.sphere.intersectsBox(box)) {
        return;
      }

      entry.faces.forEach((face: ILightShadowFace) => {
        if (isBoxInPlanes(box, face.planes)) {
          face.isStale = true;
        }
      });

      if (isAnimated) {
        this.moved.add(entry);
      }
    });
  }
}
