import { describe, expect, it } from "@jest/globals";

import { RendererPassInspector } from "#/timing/renderer-pass-inspector";

describe("RendererPassInspector", () => {
  it("tags each render with the pass around it, by the frame its uid names", () => {
    const inspector: RendererPassInspector = new RendererPassInspector();

    inspector.enter("gbuffer");
    inspector.beginRender("r:1:2:f7");
    inspector.leave();
    inspector.beginRender("r:2:1:f7");
    inspector.enter("sun");
    inspector.beginCompute("c:1:f8");
    inspector.beginRender("unframed");

    expect([...inspector.issued]).toEqual([
      [7, [["r:1:2:f7", "gbuffer"]]],
      [8, [["c:1:f8", "sun"]]],
    ]);
  });

  it("tags nothing while the device is not timing, and lets go of what waited", () => {
    const inspector: RendererPassInspector = new RendererPassInspector();

    inspector.enter("gbuffer");
    inspector.beginRender("r:1:1:f1");
    inspector.setRecording(false);
    inspector.beginRender("r:1:1:f2");

    expect(inspector.issued.size).toBe(0);
  });

  it("keeps a bounded number of frames waiting, letting the oldest go", () => {
    const inspector: RendererPassInspector = new RendererPassInspector();

    inspector.enter("gbuffer");

    for (let frame: number = 0; frame < 40; frame += 1) {
      inspector.beginRender(`r:1:1:f${frame}`);
    }

    expect(inspector.issued.size).toBe(16);
    expect(inspector.issued.keys().next().value).toBe(24);
  });

  it("lets go of the frames read", () => {
    const inspector: RendererPassInspector = new RendererPassInspector();

    inspector.enter("gbuffer");
    inspector.beginRender("r:1:1:f1");
    inspector.beginRender("r:1:1:f2");
    inspector.consume([1]);

    expect([...inspector.issued.keys()]).toEqual([2]);
  });
});
