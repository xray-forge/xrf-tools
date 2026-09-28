import { Nullable } from "@xrf/types";
import {
  attribute,
  bitcast,
  float,
  Fn,
  instanceIndex,
  select,
  uvec2,
  varying,
  vec2,
  vec3,
  vec4,
  vertexIndex,
} from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { IClusterAttribute } from "#/geometry/cluster-attribute";
import { IClusterSource, toClusterSource } from "#/geometry/cluster-source";
import { EClusterWordFormat } from "#/geometry/cluster-word-format";

// A clustered draw reads nothing through vertex buffers. Its instance is an entry of its batch's region of the view's
// list, a cluster and a place; its vertex index a corner of one of the cluster's triangles, or an end of one of their
// edges for a wireframe. The vertex is read from the cluster's run of indices in its arena, and every attribute of it
// from the arena's words.

/**
 * @param builder - A builder.
 * @returns Whether it builds for a clustered static draw, whose vertices are read from its arena.
 */
export function isClusteredBuild(builder: NodeBuilder): boolean {
  return toClusterSource(builder.geometry) !== null;
}

/**
 * @param builder - The builder of a clustered build.
 * @returns The entry the instance being drawn is: its cluster, then its place.
 */
export function toClusterEntry(builder: NodeBuilder): Node<"uvec2"> {
  const source: Nullable<IClusterSource> = toClusterSource(builder.geometry);

  // Only a clustered build draws an entry; a material staged over another layout compiles, and is never drawn so.
  return (source ? source.entryNode.element(instanceIndex) : uvec2(0, 0)) as unknown as Node<"uvec2">;
}

/**
 * The vertex a clustered build draws, in its arena: a corner of the cluster's triangle the vertex index names, those
 * past its triangles falling on its last vertex so their triangles are nothing; for a wireframe, an end of an edge,
 * those past falling on one vertex so their lines are nothing. Built once a build, however many attributes read it.
 */
const toClusterVertex = Fn((_: [], builder: NodeBuilder): Node<"uint"> => {
  const source: IClusterSource = toClusterSource(builder.geometry) as IClusterSource;
  const range = source.rangeNode.element(toClusterEntry(builder).x) as unknown as Node<"uvec4">;
  const last: Node<"uint"> = range.y.mul(3).sub(1);
  let corner: Node<"uint">;

  if ((builder.object as { isLineSegments?: boolean } | null)?.isLineSegments) {
    const triangle: Node<"uint"> = vertexIndex.div(6);
    const edge: Node<"uint"> = vertexIndex.mod(6).div(2);
    const end: Node<"uint"> = vertexIndex.mod(2);

    corner = select(triangle.lessThan(range.y), triangle.mul(3).add(edge.add(end).mod(3)), last) as Node<"uint">;
  } else {
    const index: Node<"uint"> = vertexIndex as unknown as Node<"uint">;

    corner = select(index.lessThan(last), index, last) as unknown as Node<"uint">;
  }

  return range.z.add(source.indexNode.element(range.x.add(corner)) as unknown as Node<"uint">).toVar("clusterVertex");
}).once();

/**
 * A vertex attribute of a clustered build, read from its arena's words as its vertex buffer would have read it: a
 * float's components as floats, an unsigned integer's as integers, four normalized bytes as a vector of them.
 *
 * @param builder - The builder of a clustered build.
 * @param name - The attribute.
 * @returns Its value at the vertex drawn, or null where the arena does not store it.
 */
export function toClusterAttribute(builder: NodeBuilder, name: string): Nullable<Node> {
  const source: IClusterSource = toClusterSource(builder.geometry) as IClusterSource;
  const stored: Nullable<IClusterAttribute> = source.layout.find((it: IClusterAttribute) => it.name === name) ?? null;

  if (!stored) {
    return null;
  }

  const first: Node<"uint"> = (toClusterVertex() as unknown as Node<"uint">).mul(source.stride).add(stored.offset);

  function word(component: number): Node<"uint"> {
    return source.wordNode.element(first.add(component)) as unknown as Node<"uint">;
  }

  switch (stored.format) {
    case EClusterWordFormat.UNORM8X4:
      return toUnorm8x4(word(0));

    case EClusterWordFormat.UINT:
      return stored.itemSize === 1 ? word(0) : uvec2(word(0), word(1));

    case EClusterWordFormat.FLOAT: {
      const floats: Array<Node<"float">> = Array.from(
        { length: stored.itemSize },
        (_: unknown, component: number) => bitcast(word(component), "float") as unknown as Node<"float">
      );

      switch (stored.itemSize) {
        case 1:
          return floats[0];
        case 2:
          return vec2(floats[0], floats[1]);
        case 3:
          return vec3(floats[0], floats[1], floats[2]);
        default:
          return vec4(floats[0], floats[1], floats[2], floats[3]);
      }
    }
  }
}

/**
 * A vertex attribute for whichever build reads it: a clustered one's from its arena, every other's through its vertex
 * buffer. A clustered build's read is the vertex stage's: a float one reaches the fragment stage through a varying, as
 * a vertex buffer's does; words are decoded in the vertex stage, since an integer varying is not interpolated.
 *
 * @param name - The attribute.
 * @param type - Its type, as the vertex buffer's read would give it.
 * @returns The attribute.
 */
export function toVertexAttribute<T extends string>(name: string, type: T): Node<T> {
  return Fn((_: [], builder: NodeBuilder): Node<T> => {
    const clustered: Nullable<Node> = isClusteredBuild(builder) ? toClusterAttribute(builder, name) : null;

    if (!clustered) {
      return attribute<T>(name, type);
    }

    return (type.startsWith("u") || type.startsWith("i") ? clustered : varying(clustered)) as Node<T>;
  })() as unknown as Node<T>;
}

/** Four bytes of a word as the normalized vector a `unorm8x4` vertex buffer reads, its lowest byte first. */
function toUnorm8x4(word: Node<"uint">): Node<"vec4"> {
  return vec4(
    float(word.bitAnd(255)),
    float(word.shiftRight(8).bitAnd(255)),
    float(word.shiftRight(16).bitAnd(255)),
    float(word.shiftRight(24))
  ).div(255) as unknown as Node<"vec4">;
}

/**
 * @param builder - A builder.
 * @param name - A vertex attribute.
 * @returns Whether the geometry being built for carries it, in its arena for a clustered build.
 */
export function hasVertexAttribute(builder: NodeBuilder, name: string): boolean {
  const source: Nullable<IClusterSource> = toClusterSource(builder.geometry);

  return source
    ? source.layout.some((stored: IClusterAttribute) => stored.name === name)
    : Boolean(builder.geometry?.hasAttribute(name));
}
