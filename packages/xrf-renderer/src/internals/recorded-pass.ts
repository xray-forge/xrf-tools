/** A pass's commands as they replay: by method name, then their arguments. */
type TReplayedPass = Record<string, (...args: Array<unknown>) => unknown>;

/**
 * What three records a pass of the frame's shared encoder into: each command kept, by its method's name and its
 * arguments, in one array kept between passes, and replayed into the real pass as it ends, once the writes it reads are
 * in front of it. A command's trailing `undefined` arguments are left off, as WebGPU's overloads leave them. One is open
 * at a time, so one serves every pass.
 */
export class RecordedPass {
  public label: string = "";

  private readonly ops: Array<unknown> = [];
  /** A command's arguments as it is recorded, kept between commands. */
  private readonly args: Array<unknown> = [undefined, undefined, undefined, undefined, undefined, undefined];
  private readonly onEnd: () => void;

  /**
   * @param onEnd - Told as three ends the pass, which then replays it.
   */
  public constructor(onEnd: () => void) {
    this.onEnd = onEnd;
  }

  /** Forgets every command, for the next pass. */
  public reset(): void {
    this.ops.length = 0;
    this.label = "";
  }

  /**
   * @param pass - The real pass, begun: every command recorded is issued on it in order.
   */
  public replay(pass: object): void {
    const ops: Array<unknown> = this.ops;
    const target = pass as TReplayedPass;

    for (let at: number = 0; at < ops.length;) {
      const method: (...args: Array<unknown>) => unknown = target[ops[at] as string];
      const count: number = ops[at + 1] as number;

      switch (count) {
        case 0:
          method.call(pass);
          break;
        case 1:
          method.call(pass, ops[at + 2]);
          break;
        case 2:
          method.call(pass, ops[at + 2], ops[at + 3]);
          break;
        case 3:
          method.call(pass, ops[at + 2], ops[at + 3], ops[at + 4]);
          break;
        case 4:
          method.call(pass, ops[at + 2], ops[at + 3], ops[at + 4], ops[at + 5]);
          break;
        case 5:
          method.call(pass, ops[at + 2], ops[at + 3], ops[at + 4], ops[at + 5], ops[at + 6]);
          break;
        default:
          method.call(pass, ops[at + 2], ops[at + 3], ops[at + 4], ops[at + 5], ops[at + 6], ops[at + 7]);
      }

      at += 2 + count;
    }
  }

  public setPipeline(pipeline: unknown): void {
    this.record("setPipeline", pipeline);
  }

  public setBindGroup(index: unknown, group: unknown, offsets?: unknown, start?: unknown, length?: unknown): void {
    this.record("setBindGroup", index, group, offsets, start, length);
  }

  public setVertexBuffer(slot: unknown, buffer: unknown, offset?: unknown, size?: unknown): void {
    this.record("setVertexBuffer", slot, buffer, offset, size);
  }

  public setIndexBuffer(buffer: unknown, format: unknown, offset?: unknown, size?: unknown): void {
    this.record("setIndexBuffer", buffer, format, offset, size);
  }

  public draw(vertices: unknown, instances?: unknown, firstVertex?: unknown, firstInstance?: unknown): void {
    this.record("draw", vertices, instances, firstVertex, firstInstance);
  }

  public drawIndexed(
    indices: unknown,
    instances?: unknown,
    firstIndex?: unknown,
    baseVertex?: unknown,
    firstInstance?: unknown
  ): void {
    this.record("drawIndexed", indices, instances, firstIndex, baseVertex, firstInstance);
  }

  public drawIndirect(buffer: unknown, offset: unknown): void {
    this.record("drawIndirect", buffer, offset);
  }

  public drawIndexedIndirect(buffer: unknown, offset: unknown): void {
    this.record("drawIndexedIndirect", buffer, offset);
  }

  public executeBundles(bundles: unknown): void {
    this.record("executeBundles", bundles);
  }

  public setViewport(x: unknown, y: unknown, width: unknown, height: unknown, min: unknown, max: unknown): void {
    this.record("setViewport", x, y, width, height, min, max);
  }

  public setScissorRect(x: unknown, y: unknown, width: unknown, height: unknown): void {
    this.record("setScissorRect", x, y, width, height);
  }

  public setBlendConstant(color: unknown): void {
    this.record("setBlendConstant", color);
  }

  public setStencilReference(reference: unknown): void {
    this.record("setStencilReference", reference);
  }

  public beginOcclusionQuery(index: unknown): void {
    this.record("beginOcclusionQuery", index);
  }

  public endOcclusionQuery(): void {
    this.record("endOcclusionQuery");
  }

  public dispatchWorkgroups(x: unknown, y?: unknown, z?: unknown): void {
    this.record("dispatchWorkgroups", x, y, z);
  }

  public dispatchWorkgroupsIndirect(buffer: unknown, offset: unknown): void {
    this.record("dispatchWorkgroupsIndirect", buffer, offset);
  }

  public pushDebugGroup(label: unknown): void {
    this.record("pushDebugGroup", label);
  }

  public popDebugGroup(): void {
    this.record("popDebugGroup");
  }

  public insertDebugMarker(label: unknown): void {
    this.record("insertDebugMarker", label);
  }

  public end(): void {
    this.onEnd();
  }

  private record(method: string, a?: unknown, b?: unknown, c?: unknown, d?: unknown, e?: unknown, f?: unknown): void {
    const { args, ops } = this;

    args[0] = a;
    args[1] = b;
    args[2] = c;
    args[3] = d;
    args[4] = e;
    args[5] = f;

    let count: number = args.length;

    while (count > 0 && args[count - 1] === undefined) {
      count -= 1;
    }

    ops.push(method, count);

    for (let at: number = 0; at < count; at += 1) {
      ops.push(args[at]);
    }
  }
}
