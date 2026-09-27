import { atomicAdd, Fn, If, instanceIndex, mat4, select, storage, uint, uniformArray, uvec2, vec4 } from "three/tsl";
import { ComputeNode, Node, StorageBufferNode, UniformArrayNode, Vector4 } from "three/webgpu";

import { toInFrustum } from "#/scene/static/static-frustum.tsl";
import { createLodCullShader } from "#/scene/static/static-lod-cull.tsl";
import { toOccluded } from "#/scene/static/static-occlusion.tsl";
import { toBandDrawn, toFinestBand, toImpostorRow, toLodDrawn } from "#/scene/static/static-row-tests.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import {
  EStaticPool,
  EStaticSlotKind,
  EStaticView,
  STATIC_BATCH_ARGUMENTS,
  STATIC_CLUSTER_VERTICES,
  STATIC_CLUSTER_WIRE_VERTICES,
  STATIC_CULL_COUNTS,
  STATIC_LIGHT_VIEW_START,
  STATIC_NO_BATCH,
  STATIC_PLACE_COLUMNS,
  STATIC_SHADOW_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/**
 * The culls of the static draws, a frame's worth, each built over the buffers as they are laid out and dispatched only
 * as far as their clusters, rows and batches are used.
 */
export interface IStaticCullShader {
  /**
   * The camera's first view, in dispatch order: its arguments and the second view's cleared, the LOD cull, then its
   * single draws' clusters, then its instanced draws' rows and their clusters.
   */
  early: ReadonlyArray<ComputeNode>;
  /** The camera's second view: what the first left as hidden, tested against this frame's depth. */
  late: ComputeNode;
  /** Each of the camera's views' arguments rewritten for its wireframe draw, the first then the second. */
  wire: ReadonlyArray<ComputeNode>;
  /** Six planes, normals pointing in, `w` the constant. */
  planes: ReadonlyArray<Vector4>;
  /** Each shadow view's cull: its arguments cleared, its single draws' clusters, its rows', and the planes they read. */
  views: ReadonlyArray<IStaticViewCullShader>;
}

/** One shadow view's cull: no occlusion and no second phase, only its frustum. */
export interface IStaticViewCullShader {
  cull: ReadonlyArray<ComputeNode>;
  planes: ReadonlyArray<Vector4>;
}

/** The nodes one cull reads and writes the buffers through, its own: an atomic node is not a read-only one. */
interface ICullNodes {
  slots: StorageBufferNode<"uvec4">;
  ranges: StorageBufferNode<"uvec4">;
  spheres: StorageBufferNode<"vec4">;
  places: StorageBufferNode<"vec4">;
  regions: StorageBufferNode<"uvec4">;
  args: StorageBufferNode<"uint">;
  lists: StorageBufferNode<"uvec2">;
}

/** What the camera's first view writes besides: what it leaves for the second, the depth it tests by, what it counts. */
interface ICameraNodes {
  candidates: StorageBufferNode<"uvec2">;
  candidateCount: StorageBufferNode<"uint">;
  pyramid: StorageBufferNode<"float">;
  counts: StorageBufferNode<"uint">;
}

/**
 * @param buffers - The static draw buffers, as they are laid out now.
 * @returns Every cull, and the planes they read.
 */
export function createStaticCullShader(buffers: StaticDrawBuffers): IStaticCullShader {
  const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  const planeNodes = uniformArray(planes, "vec4");

  return {
    early: [
      createClearShader(buffers, EStaticView.EARLY),
      createClearShader(buffers, EStaticView.LATE),
      createLodCullShader(buffers),
      createSingleShader(buffers, EStaticView.EARLY, planeNodes),
      createRowShader(buffers, EStaticView.EARLY, planeNodes),
    ],
    late: createLateShader(buffers),
    planes,
    wire: [createWireShader(buffers, 0), createWireShader(buffers, 1)],
    views: Array.from({ length: STATIC_SHADOW_VIEWS }, (_, shadow: number) => {
      const view: number = EStaticView.SHADOW + shadow;
      const viewPlanes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());
      const viewPlaneNodes = uniformArray(viewPlanes, "vec4");

      return {
        cull: [
          createClearShader(buffers, view),
          createSingleShader(buffers, view, viewPlaneNodes),
          createRowShader(buffers, view, viewPlaneNodes),
        ],
        planes: viewPlanes,
      };
    }),
  };
}

/** A view's own nodes over the buffers, its arguments atomic. */
function createNodes(buffers: StaticDrawBuffers, view: number): ICullNodes {
  const batches: number = buffers.capacity(EStaticPool.BATCHES);
  const clusters: number = buffers.capacity(EStaticPool.CLUSTERS);

  return {
    args: storage(buffers.viewArgs[view], "uint", batches * STATIC_BATCH_ARGUMENTS).toAtomic(),
    lists: storage(buffers.lists, "uvec2", buffers.toListBase(buffers.viewArgs.length)),
    places: storage(buffers.places, "vec4", buffers.capacity(EStaticPool.PLACES) * STATIC_PLACE_COLUMNS).toReadOnly(),
    ranges: storage(buffers.clusterRanges, "uvec4", clusters).toReadOnly(),
    regions: storage(buffers.batchRegions, "uvec4", batches).toReadOnly(),
    slots: storage(buffers.slots, "uvec4", buffers.capacity(EStaticPool.SLOTS) * 2).toReadOnly(),
    spheres: storage(buffers.clusterSpheres, "vec4", clusters).toReadOnly(),
  };
}

function createCameraNodes(buffers: StaticDrawBuffers): ICameraNodes {
  return {
    candidateCount: storage(buffers.candidateCount, "uint", 1).toAtomic(),
    candidates: storage(buffers.candidates, "uvec2", buffers.capacity(EStaticPool.SURFACE_LIST)),
    counts: storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic(),
    pyramid: storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly(),
  };
}

/**
 * Lists a cluster in its place in its batch's region of a view's list, by the next instance of its batch's draw. A
 * region holds every entry its batch's slots can list, so the count never passes it.
 */
function toAppended(
  nodes: ICullNodes,
  base: number,
  batch: Node<"uint">,
  cluster: Node<"uint">,
  place: Node<"uint">
): void {
  const region = nodes.regions.element(batch) as unknown as Node<"uvec4">;
  const kept = atomicAdd(nodes.args.element(batch.mul(STATIC_BATCH_ARGUMENTS).add(1)), uint(1));

  If(kept.lessThan(region.y), () => {
    nodes.lists.element(uint(base).add(region.x).add(kept)).assign(uvec2(cluster, place));
  });
}

/** Leaves a cluster in its place for the camera's second view, which tests it against this frame's depth. */
function toLeft(camera: ICameraNodes, cluster: Node<"uint">, place: Node<"uint">): void {
  const at = atomicAdd(camera.candidateCount.element(0), uint(1));

  camera.candidates.element(at).assign(uvec2(cluster, place));
}

/** A cluster's sphere in its mesh's own space, stood in a place: its centre by the matrix, its radius by its scale. */
function toPlacedSphere(local: Node<"vec4">, place: Node<"uint">, places: StorageBufferNode<"vec4">): Node<"vec4"> {
  const first: Node<"uint"> = place.mul(STATIC_PLACE_COLUMNS);
  const [x, y, z, w] = [0, 1, 2, 3].map((column: number) => places.element(first.add(column)));
  const matrix = mat4(x, y, z, w) as unknown as Node<"mat4">;
  const scale = (places.element(first.add(4)) as unknown as Node<"vec4">).w;

  return vec4(matrix.mul(vec4(local.xyz, 1)).xyz, local.w.mul(scale)) as unknown as Node<"vec4">;
}

/**
 * Clears a view's arguments, one invocation a batch: none of its clusters kept yet, drawn from its region of the
 * view's list. The camera's first view also starts what it leaves for the second at none.
 */
function createClearShader(buffers: StaticDrawBuffers, view: number): ComputeNode {
  const batches: number = buffers.capacity(EStaticPool.BATCHES);
  const args = storage(buffers.viewArgs[view], "uint", batches * STATIC_BATCH_ARGUMENTS);
  const regions = storage(buffers.batchRegions, "uvec4", batches).toReadOnly();
  const candidateCount = storage(buffers.candidateCount, "uint", 1);
  const base: number = buffers.toListBase(view);

  return Fn(() => {
    const region = regions.element(instanceIndex) as unknown as Node<"uvec4">;
    const at = instanceIndex.mul(STATIC_BATCH_ARGUMENTS);

    args.element(at).assign(select(region.y.greaterThan(0), uint(STATIC_CLUSTER_VERTICES), uint(0)));
    args.element(at.add(1)).assign(0);
    args.element(at.add(2)).assign(0);
    args.element(at.add(3)).assign(uint(base).add(region.x));

    if (view === EStaticView.EARLY) {
      If(instanceIndex.equal(0), () => {
        candidateCount.element(0).assign(0);
      });
    }
  })().compute(batches);
}

/**
 * A view's cull of the single draws' clusters, one invocation a cluster: a cluster of a single draw its view's batch
 * draws is kept where its sphere reaches into the view. The camera's first view leaves one the last frame's depth
 * hides for its second. An instanced draw's clusters are its rows'.
 */
function createSingleShader(buffers: StaticDrawBuffers, view: number, planes: UniformArrayNode<string>): ComputeNode {
  const clusters: number = buffers.capacity(EStaticPool.CLUSTERS);
  const nodes: ICullNodes = createNodes(buffers, view);
  const camera: ICameraNodes = createCameraNodes(buffers);
  const base: number = buffers.toListBase(view);
  const isCamera: boolean = view === EStaticView.EARLY;

  return Fn(() => {
    const range = nodes.ranges.element(instanceIndex) as unknown as Node<"uvec4">;
    const head = nodes.slots.element(range.w.mul(2)) as unknown as Node<"uvec4">;
    const tail = nodes.slots.element(range.w.mul(2).add(1)) as unknown as Node<"uvec4">;
    const batch: Node<"uint"> = isCamera ? tail.x : tail.y;

    If(range.y.greaterThan(0).and(head.w.equal(EStaticSlotKind.SINGLE)).and(batch.notEqual(STATIC_NO_BATCH)), () => {
      const sphere = nodes.spheres.element(instanceIndex) as unknown as Node<"vec4">;

      If(toInFrustum(sphere, planes).equal(1), () => {
        if (!isCamera) {
          toAppended(nodes, base, batch, instanceIndex, head.z);

          return;
        }

        If(toOccluded(sphere, buffers.occlusion.previous, camera.pyramid, buffers.occlusion).equal(1), () => {
          toLeft(camera, instanceIndex, head.z);
        }).Else(() => {
          toAppended(nodes, base, batch, instanceIndex, head.z);
          atomicAdd(camera.counts.element(0), uint(1));
          atomicAdd(camera.counts.element(1), range.y);
        });
      });
    });
  })().compute(clusters);
}

/**
 * A view's cull of the instanced draws, one invocation a row: a row whose place its view draws, at the band its detail
 * picks, and which reaches into the view, keeps every cluster of its draw's that reaches into it too, stood in the
 * row's place. The camera's first view leaves what the last frame's depth hides, the whole row's or a cluster's, for
 * its second. A shadow view casts every clump as its trees and never as its impostor (`add_leafs_static`), and a
 * progressive tree into a cascade at the band its detail picks, as it draws, and into a light's face at its finest.
 */
function createRowShader(buffers: StaticDrawBuffers, view: number, planes: UniformArrayNode<string>): ComputeNode {
  const rows: number = buffers.capacity(EStaticPool.ROWS);
  const nodes: ICullNodes = createNodes(buffers, view);
  const camera: ICameraNodes = createCameraNodes(buffers);
  const rowLods = storage(buffers.rowLods, "uvec2", rows).toReadOnly();
  const rowSpheres = storage(buffers.rowSpheres, "vec4", rows).toReadOnly();
  const rowTargets = storage(buffers.rowTargets, "uvec4", rows).toReadOnly();
  const lodTerms = storage(buffers.lodTerms, "uvec4", buffers.capacity(EStaticPool.LODS)).toReadOnly();
  const base: number = buffers.toListBase(view);
  const isCamera: boolean = view === EStaticView.EARLY;

  return Fn(() => {
    const sphere = rowSpheres.element(instanceIndex) as unknown as Node<"vec4">;
    const words = rowLods.element(instanceIndex) as unknown as Node<"uvec2">;
    const target = rowTargets.element(instanceIndex) as unknown as Node<"uvec4">;
    const isDrawn: Node<"bool"> = isCamera
      ? toLodDrawn(words.x, lodTerms).and(toBandDrawn(words.y, sphere, buffers.lod))
      : toImpostorRow(words.x)
          .not()
          .and(
            view - EStaticView.SHADOW >= STATIC_LIGHT_VIEW_START
              ? toFinestBand(words.y)
              : toBandDrawn(words.y, sphere, buffers.lod)
          );

    If(isDrawn.and(toInFrustum(sphere, planes).equal(1)), () => {
      const head = nodes.slots.element(target.y.mul(2)) as unknown as Node<"uvec4">;
      const tail = nodes.slots.element(target.y.mul(2).add(1)) as unknown as Node<"uvec4">;
      const batch: Node<"uint"> = isCamera ? tail.x : tail.y;
      const place: Node<"uint"> = target.x;

      If(head.w.equal(EStaticSlotKind.LISTED).and(batch.notEqual(STATIC_NO_BATCH)), () => {
        const isRowHidden = isCamera
          ? toOccluded(sphere, buffers.occlusion.previous, camera.pyramid, buffers.occlusion).toVar()
          : uint(0);

        loopNamed({ end: head.y, name: "clusterOfRow", start: uint(0), type: "uint" }, (index: Node<"uint">) => {
          const cluster: Node<"uint"> = head.x.add(index);
          const placed = toPlacedSphere(
            nodes.spheres.element(cluster) as unknown as Node<"vec4">,
            place,
            nodes.places
          ).toVar();

          If(toInFrustum(placed, planes).equal(1), () => {
            if (!isCamera) {
              toAppended(nodes, base, batch, cluster, place);

              return;
            }

            If(
              isRowHidden
                .equal(1)
                .or(toOccluded(placed, buffers.occlusion.previous, camera.pyramid, buffers.occlusion).equal(1)),
              () => {
                toLeft(camera, cluster, place);
              }
            ).Else(() => {
              toAppended(nodes, base, batch, cluster, place);
              atomicAdd(camera.counts.element(0), uint(1));
              atomicAdd(camera.counts.element(1), (nodes.ranges.element(cluster) as unknown as Node<"uvec4">).y);
            });
          });
        });
      });
    });
  })().compute(rows);
}

/**
 * The camera's second view, one invocation a cluster the first left: kept where this frame's depth so far does not
 * hide it, and counted as occluded where it still does.
 */
function createLateShader(buffers: StaticDrawBuffers): ComputeNode {
  const capacity: number = buffers.capacity(EStaticPool.SURFACE_LIST);
  const nodes: ICullNodes = createNodes(buffers, EStaticView.LATE);
  const candidates = storage(buffers.candidates, "uvec2", capacity).toReadOnly();
  const candidateCount = storage(buffers.candidateCount, "uint", 1).toReadOnly();
  const pyramid = storage(buffers.pyramid, "float", buffers.capacity(EStaticPool.PYRAMID)).toReadOnly();
  const counts = storage(buffers.counts, "uint", STATIC_CULL_COUNTS).toAtomic();
  const base: number = buffers.toListBase(EStaticView.LATE);

  return Fn(() => {
    If(instanceIndex.lessThan(candidateCount.element(0)), () => {
      const entry = candidates.element(instanceIndex) as unknown as Node<"uvec2">;
      const range = nodes.ranges.element(entry.x) as unknown as Node<"uvec4">;
      const head = nodes.slots.element(range.w.mul(2)) as unknown as Node<"uvec4">;
      const tail = nodes.slots.element(range.w.mul(2).add(1)) as unknown as Node<"uvec4">;
      const local = nodes.spheres.element(entry.x) as unknown as Node<"vec4">;
      const sphere = select(
        head.w.equal(EStaticSlotKind.SINGLE),
        local,
        toPlacedSphere(local, entry.y, nodes.places)
      ) as unknown as Node<"vec4">;

      If(toOccluded(sphere, buffers.occlusion.current, pyramid, buffers.occlusion).equal(0), () => {
        toAppended(nodes, base, tail.x, entry.x, entry.y);
        atomicAdd(counts.element(0), uint(1));
        atomicAdd(counts.element(1), range.y);
      }).Else(() => {
        atomicAdd(counts.element(2), uint(1));
        atomicAdd(counts.element(3), range.y);
      });
    });
  })().compute(capacity);
}

/**
 * Rewrites one of the camera's views' arguments as its wireframe draw's, one invocation a batch: a cluster's edges
 * twice its triangles' vertices, the rest as the cull left them.
 */
function createWireShader(buffers: StaticDrawBuffers, phase: number): ComputeNode {
  const batches: number = buffers.capacity(EStaticPool.BATCHES);
  const source = storage(buffers.viewArgs[phase], "uint", batches * STATIC_BATCH_ARGUMENTS).toReadOnly();
  const target = storage(buffers.wireArgs[phase], "uint", batches * STATIC_BATCH_ARGUMENTS);

  return Fn(() => {
    const at = instanceIndex.mul(STATIC_BATCH_ARGUMENTS);

    target.element(at).assign(select(source.element(at).greaterThan(0), uint(STATIC_CLUSTER_WIRE_VERTICES), uint(0)));
    target.element(at.add(1)).assign(source.element(at.add(1)));
    target.element(at.add(2)).assign(0);
    target.element(at.add(3)).assign(source.element(at.add(3)));
  })().compute(batches);
}
