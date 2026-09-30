import { Nullable } from "@xrf/types";
import { WGSLNodeBuilder } from "three/webgpu";

/** The kinds of uniform three declares as a buffer binding. */
const BUFFER_TYPES: ReadonlySet<string> = new Set(["buffer", "storageBuffer", "indirectStorageBuffer"]);

/** What three names a buffer of no name of its own: after its uniform's id, which counts every uniform ever made. */
const UNNAMED_BUFFER_PREFIX: string = "NodeBuffer_";

/** A uniform as a builder hands it out: its id, and the name its declaration and every read of it take. */
interface IBufferUniform {
  id: number;
  name: string;
}

type TGetUniformFromNode = (
  this: object,
  node: unknown,
  type: string,
  shaderStage: string,
  name?: Nullable<string>
) => IBufferUniform;

/** How many buffers each builder has named so far. */
const namedBuffers: WeakMap<object, number> = new WeakMap();
let isAdopted: boolean = false;

/**
 * Names each buffer three would name after its uniform's id by its order in the shader built instead, so a shader's
 * WGSL is the same from run to run and the browser's pipeline cache, keyed by it, finds what an earlier run compiled.
 */
export function adoptStableBufferNames(): void {
  if (isAdopted) {
    return;
  }

  isAdopted = true;

  const prototype: { getUniformFromNode: TGetUniformFromNode } = WGSLNodeBuilder.prototype as unknown as {
    getUniformFromNode: TGetUniformFromNode;
  };
  const getUniformFromNode: TGetUniformFromNode = prototype.getUniformFromNode;

  prototype.getUniformFromNode = function (node, type, shaderStage, name = null): IBufferUniform {
    const uniform: IBufferUniform = getUniformFromNode.call(this, node, type, shaderStage, name);

    nameBufferStably(this, uniform, type, name);

    return uniform;
  };
}

/**
 * @param builder - The builder the uniform is from: one per shader a material or compute node builds.
 * @param uniform - The uniform it handed out.
 * @param type - The uniform's kind.
 * @param name - The name it was asked for under, if any.
 */
export function nameBufferStably(builder: object, uniform: IBufferUniform, type: string, name: Nullable<string>): void {
  // Asked for by name, or named already: this builder handed it out before, or its node named it.
  if (name || !BUFFER_TYPES.has(type) || uniform.name !== `${UNNAMED_BUFFER_PREFIX}${uniform.id}`) {
    return;
  }

  const index: number = namedBuffers.get(builder) ?? 0;

  namedBuffers.set(builder, index + 1);
  // Three's own names for the rest are counted the same way: `nodeUniform0`, `nodeVar0`.
  uniform.name = `nodeBuffer${index}`;
}
