import { describe, expect, it } from "@jest/globals";
import { BufferAttribute } from "three/webgpu";

import { queueBufferUpload } from "#/scene/buffer-upload";

describe("queueBufferUpload", () => {
  // A cascade's arguments go up only when the cascade is drawn, every second to eighth frame: two changes in between
  // are both still to send, and replacing the first sent the GPU nothing of it.
  it("keeps a run still queued beside a new one, so nothing queued before the buffer's next use is lost", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);
    const version: number = attribute.version;

    queueBufferUpload(attribute, 10, 5);
    queueBufferUpload(attribute, 40, 10);

    expect(attribute.updateRanges).toEqual([
      { count: 5, start: 10 },
      { count: 10, start: 40 },
    ]);
    expect(attribute.version).toBeGreaterThan(version);
  });

  it("joins the runs still queued that a new one overlaps or touches, whoever queued them, and no others", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);

    attribute.addUpdateRange(60, 10);
    attribute.addUpdateRange(5, 2);
    attribute.addUpdateRange(20, 5);
    queueBufferUpload(attribute, 7, 14);

    expect(attribute.updateRanges).toEqual([
      { count: 10, start: 60 },
      { count: 20, start: 5 },
    ]);
  });

  // Three leaves the ranges in place where it creates a buffer, which sends all of it: one queued before stays.
  it("sends a run queued before three created the buffer again as itself, not the span up to the next", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(1000), 1);

    queueBufferUpload(attribute, 0, 4);
    queueBufferUpload(attribute, 900, 4);

    expect(attribute.updateRanges.reduce((sent: number, { count }: { count: number }) => sent + count, 0)).toBe(8);
  });

  it("queues a run of its own once three sent and cleared the last", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);

    queueBufferUpload(attribute, 10, 5);
    attribute.clearUpdateRanges();
    queueBufferUpload(attribute, 40, 10);

    expect(attribute.updateRanges).toEqual([{ count: 10, start: 40 }]);
  });
});
