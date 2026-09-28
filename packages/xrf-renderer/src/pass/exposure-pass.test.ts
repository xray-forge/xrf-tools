import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { uniform } from "three/tsl";
import { ComputeNode, Node } from "three/webgpu";

import { ExposurePass } from "#/pass/exposure-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ExposureUniforms } from "#/uniforms/exposure-uniforms";

describe("ExposurePass", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  // Three keeps a compute's pipeline and bindings until the compute's own dispose, and every join makes two more.
  it("lets both its computes go as it leaves the frame, and leaves the exposure at one", () => {
    const disposed: Array<Node> = [];

    jest.spyOn(Node.prototype, "dispose").mockImplementation(function (this: Node): void {
      disposed.push(this);
    });

    const exposure: ExposureUniforms = new ExposureUniforms(uniform(1));
    const pass: ExposurePass = new ExposurePass(new RendererTargets(), exposure);

    (exposure.adapted.array as Float32Array)[0] = 3;
    pass.dispose();

    expect(disposed.filter((node: Node) => node instanceof ComputeNode)).toHaveLength(2);
    expect((exposure.adapted.array as Float32Array)[0]).toBe(1);
  });
});
