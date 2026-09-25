import { describe, expect, it } from "@jest/globals";
import { Box3, Vector3 } from "three/webgpu";

import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

const BOX: Box3 = new Box3(new Vector3(0, 0, 0), new Vector3(1, 1, 1));
const OTHER: Box3 = new Box3(new Vector3(5, 0, 0), new Vector3(6, 1, 1));

describe("StaticShadowChanges", () => {
  it("logs where a casting slot came, went or cut out anew, and nothing for a slot that casts nothing", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, false);
    changes.put(2, OTHER, false, false);

    const seen: number = changes.version;

    changes.touch(2);
    changes.touch(1);
    changes.withdraw(1);
    changes.put(3, null, true, false);

    expect(changes.since(0)).toEqual([BOX, BOX, BOX, null]);
    expect(changes.since(seen)).toEqual([BOX, BOX, null]);
    expect(changes.since(changes.version)).toEqual([]);
  });

  it("moves a slot put again from its old box to its new one", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, false);
    changes.put(1, OTHER, true, false);

    expect(changes.since(1)).toEqual([BOX, OTHER]);
  });

  it("answers null for a version older than the log keeps, which stands for a change anywhere", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    for (let slot: number = 0; slot < 5000; slot += 1) {
      changes.put(slot, BOX, true, false);
    }

    expect(changes.since(0)).toBeNull();
    expect(changes.since(changes.version - 1)).toEqual([BOX]);
  });

  it("keeps the boxes of the casters that sway, versioned as they come and go", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, BOX, true, true);
    changes.put(2, OTHER, true, false);

    const version: number = changes.swayingVersion;

    expect([...changes.swayingBoxes]).toEqual([BOX]);

    changes.withdraw(1);

    expect([...changes.swayingBoxes]).toEqual([]);
    expect(changes.swayingVersion).toBeGreaterThan(version);
  });
});
