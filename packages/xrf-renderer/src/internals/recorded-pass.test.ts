import { describe, expect, it } from "@jest/globals";

import { RecordedPass } from "#/internals/recorded-pass";

describe("RecordedPass", () => {
  // WebGPU picks `setBindGroup`'s overload by how many arguments it is given: an `undefined` offsets is not a typed array.
  it("replays every command in order, its trailing undefined arguments left off", () => {
    const pass: RecordedPass = new RecordedPass(() => {});
    const calls: Array<Array<unknown>> = [];
    const real = {
      draw: (...args: Array<unknown>) => calls.push(["draw", ...args]),
      setBindGroup: (...args: Array<unknown>) => calls.push(["setBindGroup", ...args]),
      setViewport: (...args: Array<unknown>) => calls.push(["setViewport", ...args]),
    };
    const group = {};

    pass.setBindGroup(0, group, undefined);
    pass.setViewport(1, 2, 3, 4, 0, 1);
    pass.draw(6, undefined, 2);
    pass.replay(real);

    expect(calls).toEqual([
      ["setBindGroup", 0, group],
      ["setViewport", 1, 2, 3, 4, 0, 1],
      ["draw", 6, undefined, 2],
    ]);
  });

  it("tells its owner as three ends it, and forgets its commands once reset", () => {
    let ended: number = 0;
    const pass: RecordedPass = new RecordedPass(() => (ended += 1));
    const calls: Array<string> = [];

    pass.draw(3);
    pass.end();
    pass.reset();
    pass.replay({ draw: () => calls.push("draw") });

    expect(ended).toBe(1);
    expect(calls).toEqual([]);
  });
});
