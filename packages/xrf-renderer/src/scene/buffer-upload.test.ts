import { describe, expect, it } from "@jest/globals";
import { BufferAttribute } from "three/webgpu";

import { queueBufferUpload } from "#/scene/buffer-upload";

describe("queueBufferUpload", () => {
  // A cascade's arguments go up only when the cascade is drawn, every second to eighth frame: two changes in between
  // are both still to send, and replacing the first sent the GPU nothing of it.
  it("joins a run to one still queued, so nothing queued before the buffer's next use is lost", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);
    const version: number = attribute.version;

    queueBufferUpload(attribute, 10, 5);
    queueBufferUpload(attribute, 40, 10);

    expect(attribute.updateRanges).toEqual([{ count: 40, start: 10 }]);
    expect(attribute.version).toBeGreaterThan(version);
  });

  it("joins every run still queued, whoever queued them", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);

    attribute.addUpdateRange(60, 10);
    attribute.addUpdateRange(5, 2);
    queueBufferUpload(attribute, 30, 4);

    expect(attribute.updateRanges).toEqual([{ count: 65, start: 5 }]);
  });

  it("queues a run of its own once three sent and cleared the last", () => {
    const attribute: BufferAttribute = new BufferAttribute(new Uint32Array(100), 1);

    queueBufferUpload(attribute, 10, 5);
    attribute.clearUpdateRanges();
    queueBufferUpload(attribute, 40, 10);

    expect(attribute.updateRanges).toEqual([{ count: 10, start: 40 }]);
  });
});
