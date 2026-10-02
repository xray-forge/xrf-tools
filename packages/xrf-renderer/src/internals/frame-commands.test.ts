import { describe, expect, it } from "@jest/globals";
import { WebGPURenderer } from "three/webgpu";

import { FrameCommands } from "#/internals/frame-commands";

interface INamed {
  name: string;
}

/** A pass, its methods on its prototype as WebGPU's are, logging into its encoder. */
class FakePass {
  public constructor(private readonly log: Array<string>) {}

  public draw(count: number): void {
    this.log.push(`draw ${count}`);
  }

  public end(): void {
    this.log.push("end");
  }
}

/** An encoder, likewise, recording what it was asked. */
class FakeEncoder {
  public readonly log: Array<string> = [];
  /** Each render pass's descriptor as the pass began. */
  public readonly descriptors: Array<unknown> = [];

  public constructor(public readonly label: string) {}

  public beginRenderPass(descriptor?: unknown): FakePass {
    this.descriptors.push(JSON.parse(JSON.stringify(descriptor ?? null)));
    this.log.push("render");

    return new FakePass(this.log);
  }

  public beginComputePass(): FakePass {
    this.log.push("compute");

    return new FakePass(this.log);
  }

  public copyBufferToBuffer(
    source: INamed,
    sourceOffset: number,
    target: INamed,
    targetOffset: number,
    size: number
  ): void {
    this.log.push(`copy ${source.name}@${sourceOffset} ${target.name}@${targetOffset} ${size}`);
  }

  public finish(): { from: FakeEncoder } {
    return { from: this };
  }
}

interface IFake {
  encoders: Array<FakeEncoder>;
  /** Each submit, as each command buffer's encoder logged it, and each write the queue took, in order. */
  timeline: Array<string>;
  device: {
    createCommandEncoder(descriptor?: { label?: string }): FakeEncoder;
    createBuffer(descriptor: { label: string; size: number }): INamed & { destroy(): void };
    queue: {
      submit(buffers: Array<{ from: FakeEncoder }>): void;
      writeBuffer(buffer: INamed, offset: number, data: BufferSource, dataOffset?: number, size?: number): void;
      writeTexture(): void;
      copyExternalImageToTexture(): void;
      onSubmittedWorkDone(): Promise<void>;
    };
  };
}

function createDevice(): { fake: IFake; commands: FrameCommands } {
  const timeline: Array<string> = [];
  const encoders: Array<FakeEncoder> = [];
  const device: IFake["device"] = {
    createBuffer: (descriptor: { label: string; size: number }) => ({ destroy: () => {}, name: descriptor.label }),
    createCommandEncoder: (descriptor?: { label?: string }): FakeEncoder => {
      const encoder: FakeEncoder = new FakeEncoder(descriptor?.label ?? "");

      encoders.push(encoder);

      return encoder;
    },
    queue: {
      copyExternalImageToTexture: (): void => {
        timeline.push("image");
      },
      onSubmittedWorkDone: (): Promise<void> => Promise.resolve(),
      submit: (buffers: Array<{ from: FakeEncoder }>): void => {
        timeline.push(`submit ${buffers.map((it) => `${it.from.label}[${it.from.log.join(", ")}]`).join(" ")}`);
      },
      // The bytes sent: `dataOffset` and `size` count a typed array's elements.
      writeBuffer: (
        buffer: INamed,
        offset: number,
        data: BufferSource,
        dataOffset: number = 0,
        size?: number
      ): void => {
        const element: number = (data as { BYTES_PER_ELEMENT?: number }).BYTES_PER_ELEMENT ?? 1;

        timeline.push(
          `write ${buffer.name}@${offset} ${size === undefined ? data.byteLength - dataOffset * element : size * element}`
        );
      },
      writeTexture: (): void => {
        timeline.push("texture");
      },
    },
  };
  const renderer = { backend: { device } } as unknown as WebGPURenderer;

  return { commands: FrameCommands.adopt(renderer), fake: { device, encoders, timeline } };
}

/** Records one render as three does: an encoder, a pass, the uniforms it reads written, a draw, finished, submitted. */
function render(fake: IFake, context: number, writes: Array<INamed> = []): void {
  const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: `renderContext_${context}` });
  const pass: FakePass = encoder.beginRenderPass();

  writes.forEach((buffer: INamed) => fake.device.queue.writeBuffer(buffer, 0, new Float32Array(4)));
  pass.draw(context);
  pass.end();
  fake.device.queue.submit([encoder.finish()]);
}

describe("FrameCommands", () => {
  // Each submit is a cost of the GPU process's, and each encoder and command buffer an object it frees at a collection.
  it("records a frame's renders and computes into one encoder, submitted once as the frame ends", () => {
    const { commands, fake } = createDevice();

    commands.begin();
    render(fake, 1);
    render(fake, 2);

    const compute: FakeEncoder = fake.device.createCommandEncoder({ label: "computeGroup_7" });

    compute.beginComputePass().end();
    fake.device.queue.submit([compute.finish()]);

    expect(fake.timeline).toEqual([]);

    commands.end();

    expect(fake.timeline).toEqual(["submit frame[render, draw 1, end, render, draw 2, end, compute, end]"]);
    expect(commands.segments).toBe(1);
  });

  // Three shares a render context's camera uniforms between its renders: a queue write would reach the earlier one.
  it("writes a buffer written once a pass was recorded by a copy, in front of the pass recording as it was written", () => {
    const { commands, fake } = createDevice();
    const camera: INamed = { name: "camera" };

    commands.begin();
    render(fake, 1, [camera]);
    render(fake, 2);
    render(fake, 1, [camera]);
    commands.end();

    expect(fake.timeline).toEqual([
      "write camera@0 16",
      "write frame staging@0 16",
      "submit frame[render, draw 1, end, render, draw 2, end, copy frame staging@0 camera@0 16, render, draw 1, end]",
    ]);
    expect(commands).toMatchObject({ copies: 1, segments: 1 });
  });

  it("copies a buffer written between passes at once, in front of the next", () => {
    const { commands, fake } = createDevice();
    const lights: INamed = { name: "lights" };

    commands.begin();
    fake.device.queue.writeBuffer(lights, 8, new Uint32Array(2));
    render(fake, 1);
    fake.device.queue.writeBuffer(lights, 8, new Uint32Array(4), 1, 2);
    render(fake, 2);
    commands.end();

    expect(fake.timeline).toEqual([
      "write lights@8 8",
      "write frame staging@0 8",
      "submit frame[render, draw 1, end, copy frame staging@0 lights@8 8, render, draw 2, end]",
    ]);
  });

  it("leaves an encoder made inside one of its passes three's own, its writes sent at once ahead of it", () => {
    const { commands, fake } = createDevice();
    const flip: INamed = { name: "flip" };

    commands.begin();
    render(fake, 1);

    const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: "renderContext_2" });
    const pass: FakePass = encoder.beginRenderPass();
    const mipmaps: FakeEncoder = fake.device.createCommandEncoder({ label: "mipmaps" });

    fake.device.queue.writeBuffer(flip, 0, new Uint32Array(1));
    mipmaps.beginRenderPass().end();
    fake.device.queue.submit([mipmaps.finish()]);
    pass.draw(2);
    pass.end();
    fake.device.queue.submit([encoder.finish()]);
    commands.end();

    expect(fake.timeline).toEqual([
      "write flip@0 4",
      "submit mipmaps[render, end]",
      "submit frame[render, draw 1, end, render, draw 2, end]",
    ]);
  });

  // A copy three records on the encoder itself, outside a pass, would otherwise be overtaken by the queue's write.
  it("copies a buffer written after the frame's encoder recorded a copy of its own, before any pass", () => {
    const { commands, fake } = createDevice();
    const source: INamed = { name: "source" };
    const lights: INamed = { name: "lights" };

    commands.begin();
    fake.device.createCommandEncoder({ label: "copy" }).copyBufferToBuffer(source, 0, lights, 0, 8);
    fake.device.queue.writeBuffer(lights, 0, new Uint32Array(2));
    render(fake, 1);
    commands.end();

    expect(fake.timeline).toEqual([
      "write frame staging@0 8",
      "submit frame[copy source@0 lights@0 8, copy frame staging@0 lights@0 8, render, draw 1, end]",
    ]);
  });

  it("keeps sending a nested encoder's writes at once until its own commands are submitted, whatever else is", () => {
    const { commands, fake } = createDevice();
    const flip: INamed = { name: "flip" };
    const before: FakeEncoder = fake.device.createCommandEncoder({ label: "before" });

    commands.begin();
    render(fake, 1);

    const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: "renderContext_2" });
    const pass: FakePass = encoder.beginRenderPass();
    const mipmaps: FakeEncoder = fake.device.createCommandEncoder({ label: "mipmaps" });

    // An encoder made before the frame is submitted meanwhile, which is none of the nested one's.
    fake.device.queue.submit([before.finish()]);
    fake.device.queue.writeBuffer(flip, 0, new Uint32Array(1));
    mipmaps.beginRenderPass().end();
    fake.device.queue.submit([mipmaps.finish()]);
    pass.draw(2);
    pass.end();
    fake.device.queue.submit([encoder.finish()]);
    commands.end();

    expect(fake.timeline).toEqual([
      "submit before[]",
      "write flip@0 4",
      "submit mipmaps[render, end]",
      "submit frame[render, draw 1, end, render, draw 2, end]",
    ]);
  });

  it("submits what was recorded before a texture is written, and before a wait on the GPU", async () => {
    const { commands, fake } = createDevice();

    commands.begin();
    render(fake, 1);
    fake.device.queue.writeTexture();
    render(fake, 2);
    await fake.device.queue.onSubmittedWorkDone();
    commands.end();

    expect(fake.timeline).toEqual([
      "submit frame[render, draw 1, end]",
      "texture",
      "submit frame[render, draw 2, end]",
    ]);
    expect(commands.segments).toBe(2);
  });

  // Three's mipmap and compute passes share one descriptor each, reset the moment the pass begins.
  it("begins a pass by its descriptor as it stood when three began it, though three reset it since", () => {
    const { commands, fake } = createDevice();
    const shared = {
      colorAttachments: [{ loadOp: "clear", view: "mip 1" }],
      label: "mipmap",
      timestampWrites: { beginningOfPassWriteIndex: 0 },
    };

    commands.begin();

    const encoder: FakeEncoder = fake.device.createCommandEncoder({ label: "mipmaps" });
    const pass: FakePass = encoder.beginRenderPass(shared) as FakePass;

    shared.label = "";
    shared.colorAttachments[0].view = "";
    shared.colorAttachments.length = 0;
    shared.timestampWrites.beginningOfPassWriteIndex = -1;
    pass.end();
    commands.end();

    expect(fake.encoders.find((it: FakeEncoder) => it.label === "frame")?.descriptors).toEqual([
      {
        colorAttachments: [{ loadOp: "clear", view: "mip 1" }],
        label: "mipmap",
        timestampWrites: { beginningOfPassWriteIndex: 0 },
      },
    ]);
  });

  it("hands out three's own encoders, and sends its writes, outside a frame", () => {
    const { fake } = createDevice();

    render(fake, 1, [{ name: "camera" }]);
    render(fake, 1, [{ name: "camera" }]);

    expect(fake.timeline).toEqual([
      "write camera@0 16",
      "submit renderContext_1[render, draw 1, end]",
      "write camera@0 16",
      "submit renderContext_1[render, draw 1, end]",
    ]);
  });
});
