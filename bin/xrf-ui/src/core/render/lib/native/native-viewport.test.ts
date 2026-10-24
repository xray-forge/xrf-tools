import { describe, expect, it } from "@jest/globals";
import { ERenderViewportEvent } from "@xrf/ipc/types/xrf-renderer";
import { EWorldViewportEvent } from "@xrf/ipc/types/xrf-world";

describe("NativeViewport", () => {
  // The app sends both crates' events as one untagged union, which the page tells apart by `kind` alone.
  it("hears no kind the renderer and the world both send", () => {
    const rendered: ReadonlySet<string> = new Set<string>(Object.values(ERenderViewportEvent));

    expect(Object.values(EWorldViewportEvent).filter((kind: string) => rendered.has(kind))).toEqual([]);
  });
});
