import { attribute, bitcast, float, Fn, varying, vec2 } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import {
  PACKED_BASE_QUANT,
  PACKED_LIGHTMAP_QUANT,
  PACKED_TREE_COMPONENTS,
  PACKED_TREE_QUANT,
} from "#/geometry/renderer-packed-coordinate";
import { EVertexAttribute } from "#/shader/vertex-attribute";

/**
 * @param builder - A builder.
 * @param name - A vertex attribute.
 * @returns Whether the geometry being built carries it.
 */
function hasAttribute(builder: NodeBuilder, name: EVertexAttribute): boolean {
  return Boolean(builder.geometry?.hasAttribute(name));
}

/**
 * @param builder - A builder.
 * @returns Whether it builds for a geometry carrying the engine's packed vertex rather than three's float one.
 */
export function isPackedBuild(builder: NodeBuilder): boolean {
  return hasAttribute(builder, EVertexAttribute.PACKED_NORMAL);
}

/**
 * `unpack_D3DCOLOR` and `unpack_bx2`: a packed direction's bytes, stored `z, y, x` as `(d + 1) * 127.5`, back to the
 * direction. The `unorm8x4` read is already `byte / 255`, and `byte / 127.5 - 1` is that times two less one.
 */
function toPackedDirection(name: EVertexAttribute): Node<"vec3"> {
  return attribute<"vec4">(name, "vec4").zyx.mul(2).sub(1) as Node<"vec3">;
}

/** The packed normal in the geometry's own space. */
export function toPackedNormal(): Node<"vec3"> {
  return toPackedDirection(EVertexAttribute.PACKED_NORMAL);
}

/** The packed tangent in the geometry's own space. */
export function toPackedTangent(): Node<"vec3"> {
  return toPackedDirection(EVertexAttribute.PACKED_TANGENT);
}

/** The packed binormal in the geometry's own space. */
export function toPackedBinormal(): Node<"vec3"> {
  return toPackedDirection(EVertexAttribute.PACKED_BINORMAL);
}

/** The hemisphere term the packed normal's fourth byte carries. */
export function toPackedHemi(): Node<"float"> {
  return attribute<"vec4">(EVertexAttribute.PACKED_NORMAL, "vec4").w;
}

/**
 * The two shorts a word of a coordinate carries, as `SHORT2` stores them: the first in its low half, each sign
 * extended by an arithmetic shift. Three widens a 16-bit vertex buffer to 32 bits a component as it uploads it, so a
 * coordinate's shorts travel as whole words instead, which it passes as they are.
 */
function toShorts(word: Node<"uint">): Node<"vec2"> {
  return vec2(float(toSigned(word.shiftLeft(16)).shiftRight(16)), float(toSigned(word).shiftRight(16)));
}

/** A word's bits as a signed integer, which an arithmetic shift then sign extends. */
function toSigned(bits: Node<"uint">): Node<"int"> {
  return bitcast(bits, "int") as unknown as Node<"int">;
}

/**
 * The base coordinate from its shorts, as `toPackedCoordinate` decodes it on the CPU. A baked one adds the low bytes
 * the tangent and binormal carry; a tree's adds nothing (`deffer_tree_*.vs` scales `I.tc` by `consts` alone). Which
 * it is follows the words a vertex, one for `SHORT2` and two for `SHORT4`, so the two never share a program.
 *
 * @param builder - A builder for a geometry carrying `PACKED_UV`.
 * @returns The coordinate.
 */
export function toPackedUv(builder: NodeBuilder): Node<"vec2"> {
  if (builder.geometry.getAttribute(EVertexAttribute.PACKED_UV).itemSize === PACKED_TREE_COMPONENTS / 2) {
    return toShorts(attribute<"uvec2">(EVertexAttribute.PACKED_UV, "uvec2").x).div(PACKED_TREE_QUANT);
  }

  const fraction: Node<"vec2"> =
    hasAttribute(builder, EVertexAttribute.PACKED_TANGENT) && hasAttribute(builder, EVertexAttribute.PACKED_BINORMAL)
      ? vec2(
          attribute<"vec4">(EVertexAttribute.PACKED_TANGENT, "vec4").w,
          attribute<"vec4">(EVertexAttribute.PACKED_BINORMAL, "vec4").w
        )
      : vec2(0);

  return toShorts(attribute<"uint">(EVertexAttribute.PACKED_UV, "uint")).add(fraction).div(PACKED_BASE_QUANT);
}

/** The lightmap coordinate from its shorts. */
export function toPackedUv1(): Node<"vec2"> {
  return toShorts(attribute<"uint">(EVertexAttribute.PACKED_UV1, "uint")).div(PACKED_LIGHTMAP_QUANT);
}

/**
 * The base coordinate a surface samples, packed or three's own: chosen per geometry as its program is built. A packed
 * one is decoded in the vertex stage and interpolated as floats, since an integer varying could not be.
 *
 * @param fallback - Three's coordinate, for a geometry carrying the float one.
 * @returns The coordinate.
 */
export function toBaseCoordinate(fallback: Node<"vec2">): Node<"vec2"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec2"> =>
    hasAttribute(builder, EVertexAttribute.PACKED_UV) ? varying(toPackedUv(builder)) : fallback
  )();
}

/**
 * The lightmap coordinate a surface samples, packed or three's own.
 *
 * @param fallback - Three's second coordinate.
 * @returns The coordinate.
 */
export function toLightmapCoordinate(fallback: Node<"vec2">): Node<"vec2"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec2"> =>
    hasAttribute(builder, EVertexAttribute.PACKED_UV1) ? varying(toPackedUv1()) : fallback
  )();
}
