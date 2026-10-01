import { describe, expect, it } from "@jest/globals";
import { WebGPURenderer } from "three/webgpu";

import { FrameCommands } from "#/internals/frame-commands";

/** A pass, its methods on its prototype as WebGPU's are. */
class FakePass {
  public isEnded: boolean = false;

  public end(): void {
    this.isEnded = true;
  }
}

/** An encoder, likewise, recording what it was asked. */
class FakeEncoder {
  public readonly log: Array<string> = [];

  public constructor(public readonly label: string) {}

  public beginRenderPass(): FakePass {
    this.log.push("render");

    return new FakePass();
  }

  public beginComputePass(): FakePass {
    this.log.push("compute");

    return new FakePass();
  }

  public finish(): { from: FakeEncoder } {
    return { from: this };
  }
}

interface IFakeDevice {
  encoders: Array<FakeEncoder>;
  /** Each submit's command buffers, by the encoder each was finished from, and each write, in order. */
  timeline: Array<string>;
  device: {
    createCommandEncoder(descriptor?: { label?: string }): FakeEncoder;
    queue: {
      submit(buffers: Array<{ from: FakeEncoder }>): void;
      writeBuffer(buffer: object): void;
      onSubmittedWorkDone(): Promise<void>;
    };
  };
}

function createDevice(): { fake: IFakeDevice; commands: FrameCommands } {
  const encoders: Array<FakeEncoder> = [];
  const timeline: Array<string> = [];
  const device: IFakeDevice["device"] = {
    createCommandEncoder: (descriptor?: { label?: string }): FakeEncoder => {
      const encoder: FakeEncoder = new FakeEncoder(descriptor?.label ?? "");

      encoders.push(encoder);

      return encoder;
    },
    queue: {
      onSubmittedWorkDone: (): Promise<void> => Promise.resolve(),
      submit: (buffers: Array<{ from: FakeEncoder }>): void => {
        timeline.push(`submit ${buffers.map((it) => `${it.from.label}[${it.from.log.join(",")}]`).join(" ")}`);
      },
      writeBuffer: (buffer: object): void => {
        timeline.push(`write ${(buffer as { name: string }).name}`);
      },
    },
  };
  const renderer = { backend: { device } } as unknown as WebGPURenderer;

  return { commands: FrameCommands.adopt(renderer), fake: { device, encoders, timeline } };
}

/** Records one render into its own encoder as three does: an encoder, a pass, its uniforms written, finished. */
function render(fake: IFakeDevice, context: number, writes: Array<object> = []): void {
  const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: `renderContext_${context}` });
  const pass: FakePass = encoder.beginRenderPass();

  writes.forEach((buffer: object) => fake.device.queue.writeBuffer(buffer));
  pass.end();
  fake.device.queue.submit([encoder.finish() as { from: FakeEncoder }]);
}

describe("FrameCommands", () => {
  // Each encoder, command buffer and pass is freed by the browser only at the worker's next major collection.
  it("records a frame's renders and computes into one encoder, submitted once as the frame ends", () => {
    const { commands, fake } = createDevice();

    commands.begin();
    render(fake, 1);
    render(fake, 2);

    const compute: FakeEncoder = fake.device.createCommandEncoder({ label: "computeGroup_7" });

    compute.beginComputePass().end();
    fake.device.queue.submit([compute.finish() as { from: FakeEncoder }]);

    expect(fake.timeline).toEqual([]);

    commands.end();

    expect(fake.encoders).toHaveLength(1);
    expect(fake.timeline).toEqual(["submit frame[render,render,compute]"]);
    expect(commands.segments).toBe(1);
  });

  // Three shares a render context's camera uniforms between its renders: the second's write would reach the first.
  it("starts another segment where a render context is drawn again, so each reads its own uniforms", () => {
    const { commands, fake } = createDevice();
    const camera = { name: "camera" };

    commands.begin();
    render(fake, 1, [camera]);
    render(fake, 2);
    render(fake, 1, [camera]);
    commands.end();

    expect(fake.timeline).toEqual([
      "write camera",
      "submit frame[render,render]",
      "write camera",
      "submit frame[render]",
    ]);
    expect(commands.conflicts).toBe(0);
  });

  it("submits what was recorded before a buffer read since is written again between renders", () => {
    const { commands, fake } = createDevice();
    const lights = { name: "lights" };

    commands.begin();
    fake.device.queue.writeBuffer(lights);
    render(fake, 1);
    fake.device.queue.writeBuffer(lights);
    render(fake, 2);
    commands.end();

    expect(fake.timeline).toEqual(["write lights", "submit frame[render]", "write lights", "submit frame[render]"]);
  });

  it("counts a write it cannot order: again in a pass, after a pass of the segment read the first", () => {
    const { commands, fake } = createDevice();
    const shared = { name: "shared" };

    commands.begin();
    render(fake, 1, [shared]);
    render(fake, 2, [shared]);
    commands.end();

    expect(commands.conflicts).toBe(1);
  });

  it("leaves an encoder made inside one of its passes three's own, submitted as three asks", () => {
    const { commands, fake } = createDevice();

    commands.begin();

    const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: "renderContext_1" });
    const pass: FakePass = encoder.beginRenderPass();
    const mipmaps: FakeEncoder = fake.device.createCommandEncoder({ label: "mipmaps" });

    mipmaps.beginRenderPass().end();
    fake.device.queue.submit([mipmaps.finish() as { from: FakeEncoder }]);
    pass.end();
    fake.device.queue.submit([encoder.finish() as { from: FakeEncoder }]);
    commands.end();

    expect(fake.timeline).toEqual(["submit mipmaps[render]", "submit frame[render]"]);
  });

  it("submits the frame's work before a wait on the GPU, which is to wait on it", async () => {
    const { commands, fake } = createDevice();

    commands.begin();
    render(fake, 1);
    await fake.device.queue.onSubmittedWorkDone();

    expect(fake.timeline).toEqual(["submit frame[render]"]);

    commands.end();

    expect(fake.timeline).toHaveLength(1);
  });

  it("hands out three's own encoders outside a frame", () => {
    const { fake } = createDevice();

    render(fake, 1);
    render(fake, 1);

    expect(fake.encoders.map((it: FakeEncoder) => it.label)).toEqual(["renderContext_1", "renderContext_1"]);
    expect(fake.timeline).toHaveLength(2);
  });
});
