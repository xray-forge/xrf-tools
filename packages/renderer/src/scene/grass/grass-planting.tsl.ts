import {
  atomicAdd,
  atomicLoad,
  atomicStore,
  bitAnd,
  bitOr,
  Continue,
  cross,
  dot,
  float,
  floor,
  Fn,
  If,
  instanceIndex,
  int,
  max,
  Return,
  select,
  shiftRight,
  storage,
  uint,
  vec3,
  vec4,
} from "three/tsl";
import { ComputeNode, Node, StorageBufferNode } from "three/webgpu";

import { RENDERER_GRASS_SLOT_WORDS, RENDERER_GRASS_TRIANGLE_FLOATS } from "#/contract/scene/renderer-grass";
import { IGrassBuffers } from "#/scene/grass/grass-buffers";
import { loopNamed } from "#/shader/named-loop.tsl";
import { GrassUniforms } from "#/uniforms/grass-uniforms";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** The seed `cache_Decompress` starts every one of a slot's four generators from, before its slot is mixed in. */
const SEED: number = 0x12071980;

/** `DetailSlot::ID_Empty`: a corner planting nothing. */
const EMPTY_ID: number = 0x3f;

/** `EPS`: the determinant below which a ray is taken to lie in a triangle's plane (`xrCore/math_constants.h`). */
const RAY_EPSILON: number = 0.00001;

/** `EPS_L`, which every slot's box grows by. */
const BOX_GROWTH: number = 0.001;

/** Vectors of four floats an item takes: its place and turn, then its scale, light and wave. */
export const GRASS_ITEM_VECTORS: number = 2;

/** The compute passes that plant a frame's grass, in the order they run. */
export interface IGrassPlanting {
  /** Zeroes the counts. */
  clear: ComputeNode;
  /** Plants every slot around the camera, a thread a slot, and appends what it keeps. */
  plant: ComputeNode;
  /** Lays each model's items out after the last's, and writes its draw. */
  arrange: ComputeNode;
  /** Sorts the items into each model's range. */
  scatter: ComputeNode;
}

/**
 * The engine's dither (`bwdithermap(2, dither)` over `magic4x4`), indexed `column * 16 + row` as the planting reads
 * it: `dither[(x + shift_x) % 16][(z + shift_z) % 16]`.
 *
 * @returns Its 256 thresholds.
 */
export function createGrassDither(): Uint32Array {
  const magic: ReadonlyArray<ReadonlyArray<number>> = [
    [0, 14, 3, 13],
    [11, 5, 8, 6],
    [12, 2, 15, 1],
    [7, 9, 4, 10],
  ];
  // `N = 255 / (levels - 1)` for two levels, and `magicfact = (N - 1) / 16`, in single precision as the engine has it.
  const factor: number = Math.fround(Math.fround(255 - 1) / 16);
  const dither: Uint32Array = new Uint32Array(256);

  for (let i = 0; i < 4; i++) {
    for (let j = 0; j < 4; j++) {
      for (let k = 0; k < 4; k++) {
        for (let l = 0; l < 4; l++) {
          dither[(4 * k + i) * 16 + 4 * l + j] = Math.trunc(0.5 + magic[i][j] * factor + (magic[k][l] / 16) * factor);
        }
      }
    }
  }

  return dither;
}

/**
 * @param buffers - What the grass is planted into.
 * @param capacity - Items the lists hold.
 * @returns The sorted items as a draw reads them.
 */
export function toGrassItems(buffers: IGrassBuffers, capacity: number): StorageBufferNode<"vec4"> {
  return storage(buffers.sorted, "vec4", capacity * GRASS_ITEM_VECTORS).toReadOnly() as never;
}

/**
 * The passes planting the grass, `CDetailManager::cache_Decompress` for every slot around the camera each frame: the
 * same generators seeded the same way and drawn in the same order, the same dither, the same ray cast down onto the
 * slot's own triangles, so a slot plants what the engine plants in it. Then `UpdateVisibleM`'s cull: a slot fades by
 * its distance, shrinking its tufts to nothing at `dm_fade`, a tuft too small to see is dropped, one too small to see
 * sway stands still, and anything outside the view is not kept.
 *
 * @param buffers - What the grass is planted from and into.
 * @param uniforms - Where the camera stands and what the planting is set to.
 * @param discard - `r_ssaDISCARD`, the screen area below which the engine drops what it would draw.
 * @param capacity - Items the lists hold.
 * @returns The passes.
 */
export function createGrassPlanting(
  buffers: IGrassBuffers,
  uniforms: GrassUniforms,
  discard: Node<"float">,
  capacity: number
): IGrassPlanting {
  const models: number = buffers.modelCount;
  const counts = storage(buffers.counts, "uint", models + 1).toAtomic();
  const cursors = storage(buffers.cursors, "uint", Math.max(models, 1)).toAtomic();
  const items = storage(buffers.items, "vec4", capacity * GRASS_ITEM_VECTORS);
  const itemModels = storage(buffers.itemModels, "uint", capacity);

  const clear: ComputeNode = Fn(() => {
    atomicStore(counts.element(instanceIndex), uint(0));
  })().compute(models + 1);

  const arrange: ComputeNode = Fn(() => {
    const args = storage(buffers.args, "uint", Math.max(models, 1) * STATIC_DRAW_ARGUMENTS);
    const first = uint(0).toVar();

    for (let model = 0; model < models; model++) {
      const count = (atomicLoad(counts.element(model)) as unknown as Node<"uint">).toVar();
      const at: number = model * STATIC_DRAW_ARGUMENTS;

      args.element(at).assign(uint(buffers.indexCounts[model]));
      args.element(at + 1).assign(count);
      args.element(at + 2).assign(uint(0));
      args.element(at + 3).assign(uint(0));
      args.element(at + 4).assign(first);
      atomicStore(cursors.element(model), first);
      first.addAssign(count);
    }
  })().compute(1);

  const scatter: ComputeNode = Fn(() => {
    const sorted = storage(buffers.sorted, "vec4", capacity * GRASS_ITEM_VECTORS);
    const planted = atomicLoad(counts.element(models)) as unknown as Node<"uint">;
    const total = select(planted.lessThan(uint(capacity)), planted, uint(capacity));

    If(instanceIndex.greaterThanEqual(total), () => {
      Return();
    });

    const at = (
      atomicAdd(cursors.element(itemModels.element(instanceIndex)), uint(1)) as unknown as Node<"uint">
    ).toVar();

    for (let vector = 0; vector < GRASS_ITEM_VECTORS; vector++) {
      sorted
        .element(at.mul(GRASS_ITEM_VECTORS).add(vector))
        .assign(items.element(instanceIndex.mul(GRASS_ITEM_VECTORS).add(vector)));
    }
  })().compute(capacity);

  return {
    arrange,
    clear,
    plant: createPlant(buffers, uniforms, discard, capacity, counts, items, itemModels),
    scatter,
  };
}

/** The planting of every slot around the camera, a thread a slot. */
function createPlant(
  buffers: IGrassBuffers,
  uniforms: GrassUniforms,
  discard: Node<"float">,
  capacity: number,
  counts: StorageBufferNode<"uint">,
  items: StorageBufferNode<"vec4">,
  itemModels: StorageBufferNode<"uint">
): ComputeNode {
  const models: number = buffers.modelCount;
  const grid = storage(buffers.grid, "uint", buffers.gridLength).toReadOnly();
  const slots = storage(buffers.slots, "uint", buffers.slotWords).toReadOnly();
  const bins = storage(buffers.bins, "uint", buffers.binLength).toReadOnly();
  const triangles = storage(buffers.triangles, "float", buffers.triangleFloats).toReadOnly();
  const dither = storage(buffers.dither, "uint", 256).toReadOnly();
  const shapes = storage(buffers.models, "vec4", Math.max(models, 1) * 2).toReadOnly();

  return Fn(() => {
    const reach = int(uniforms.reach).toVar();
    const line = reach.mul(2).add(1).toVar();
    const index = int(instanceIndex).toVar();

    If(index.greaterThanEqual(line.mul(line)), () => {
      Return();
    });

    // The world slot this thread plants, and its cell in the grid.
    const sx = int(uniforms.center.x).sub(reach).add(index.mod(line)).toVar();
    const sz = int(uniforms.center.y).sub(reach).add(index.div(line)).toVar();
    const cellX = sx.add(int(uniforms.grid.z)).toVar();
    const cellZ = sz.add(int(uniforms.grid.w)).toVar();

    If(
      cellX
        .lessThan(0)
        .or(cellZ.lessThan(0))
        .or(cellX.greaterThanEqual(int(uniforms.grid.x)))
        .or(cellZ.greaterThanEqual(int(uniforms.grid.y))),
      () => {
        Return();
      }
    );

    const record = grid.element(uint(cellZ.mul(int(uniforms.grid.x)).add(cellX))).toVar();

    If(record.equal(0), () => {
      Return();
    });

    const at = record.sub(1).mul(RENDERER_GRASS_SLOT_WORDS).toVar();
    const [w0, w1, w2, w3, binStart, binCount] = [0, 1, 2, 3, 4, 5].map((word: number) =>
      slots.element(at.add(word)).toVar()
    );

    const base = float(bitAnd(w0, uint(0xfff)))
      .mul(0.2)
      .sub(200)
      .toVar();
    const top = base.add(float(bitAnd(shiftRight(w0, uint(12)), uint(0xff))).mul(0.1)).toVar();
    const ids: ReadonlyArray<Node<"uint">> = [
      bitAnd(shiftRight(w0, uint(20)), uint(0x3f)),
      bitAnd(shiftRight(w0, uint(26)), uint(0x3f)),
      bitAnd(w1, uint(0x3f)),
      bitAnd(shiftRight(w1, uint(6)), uint(0x3f)),
    ].map((id) => id.toVar());
    const sun = float(bitAnd(shiftRight(w1, uint(12)), uint(0xf)))
      .div(15)
      .toVar();
    const hemi = float(bitAnd(shiftRight(w1, uint(16)), uint(0xf)))
      .div(15)
      .toVar();
    const palettes: ReadonlyArray<Node<"uint">> = [
      bitAnd(w2, uint(0xffff)),
      shiftRight(w2, uint(16)),
      bitAnd(w3, uint(0xffff)),
      shiftRight(w3, uint(16)),
    ].map((palette) => palette.toVar());

    // `vis.box`, grown by `EPS_L`.
    const minX = float(sx).mul(2).sub(BOX_GROWTH).toVar();
    const minZ = float(sz).mul(2).sub(BOX_GROWTH).toVar();
    const minY = base.sub(BOX_GROWTH).toVar();
    const maxY = top.add(BOX_GROWTH).toVar();

    // `UpdateVisibleM`: the slot's distance from the eye fades every tuft in it, by squared metres from one to
    // `dm_fade`.
    const center = vec3(float(sx).mul(2).add(1), base.add(top).mul(0.5), float(sz).mul(2).add(1));
    const offset = uniforms.eye.sub(center);
    const distance = dot(offset, offset).toVar();
    const fadeLimit = uniforms.fade.mul(uniforms.fade);

    If(distance.greaterThan(fadeLimit), () => {
      Return();
    });

    const shrink = float(1)
      .sub(select(distance.lessThan(1), float(0), distance.sub(1).div(fadeLimit.sub(1))))
      .toVar();

    const seed = uint(SEED)
      .bitXor(uint(sx.mul(sz)))
      .toVar();
    const selection = seed.toVar();
    const jitter = seed.toVar();
    const yaw = seed.toVar();
    const scale = seed.toVar();
    const steps = int(uniforms.steps).toVar();

    // Each loop names its own counter: three names every loop's `i`, so a nested one would shadow the one outside it.
    loopNamed({ condition: "<=", end: steps, name: "row", start: int(0), type: "int" }, (z: Node<"int">) => {
      loopNamed({ condition: "<=", end: steps, name: "column", start: int(0), type: "int" }, (x: Node<"int">) => {
        const shiftX = toRandom(jitter).mod(16).toVar();
        const shiftZ = toRandom(jitter).mod(16).toVar();
        const mask = uint(0).toVar();
        const selected = uint(0).toVar();

        // `InterpolateAndDither`, clamped to the last step as the engine clamps it.
        const last = steps.sub(1);
        const column = uint(select((x as Node<"int">).lessThan(last), x, last)).toVar();
        const row = uint(select((z as Node<"int">).lessThan(last), z, last)).toVar();
        const threshold = int(dither.element(column.add(shiftX).mod(16).mul(16).add(row.add(shiftZ).mod(16)))).toVar();

        ids.forEach((id: Node<"uint">, object: number) => {
          If(id.notEqual(EMPTY_ID).and(toDensity(palettes[object], column, row, steps).greaterThan(threshold)), () => {
            mask.assign(bitOr(mask, uint(1 << object)));
            selected.addAssign(1);
          });
        });

        If(selected.equal(0), () => {
          Continue();
        });

        const pick = uint(0).toVar();

        If(selected.greaterThan(1), () => {
          pick.assign(toRandom(selection).mod(selected));
        });

        const chosen = uint(0).toVar();
        const seen = uint(0).toVar();

        for (let object = 0; object < 4; object++) {
          If(bitAnd(mask, uint(1 << object)).notEqual(0), () => {
            If(seen.equal(pick), () => {
              chosen.assign(object);
            });
            seen.addAssign(1);
          });
        }

        const model = select(
          chosen.equal(0),
          ids[0],
          select(chosen.equal(1), ids[1], select(chosen.equal(2), ids[2], ids[3]))
        ).toVar();

        If(model.greaterThanEqual(uint(models)), () => {
          Continue();
        });

        // `Item_P.set(rx + randFs, vMax.y, rz + randFs)`: MSVC evaluates the arguments right to left, so `z` draws
        // first.
        const jitterZ = toSignedRandom(jitter, uniforms.jitter).toVar();
        const jitterX = toSignedRandom(jitter, uniforms.jitter).toVar();
        const px = float(x).div(float(steps)).mul(2).add(minX).add(jitterX).toVar();
        const pz = float(z).div(float(steps)).mul(2).add(minZ).add(jitterZ).toVar();
        const y = minY.sub(5).toVar();

        loopNamed(
          { end: binStart.add(binCount), name: "entry", start: binStart, type: "uint" },
          (entry: Node<"uint">) => {
            const first = bins.element(entry).mul(RENDERER_GRASS_TRIANGLE_FLOATS).toVar();
            const [p0, p1, p2] = [0, 1, 2].map((corner: number) =>
              vec3(
                triangles.element(first.add(corner * 3)),
                triangles.element(first.add(corner * 3 + 1)),
                triangles.element(first.add(corner * 3 + 2))
              )
            );
            const range = toRayRange(vec3(px, maxY, pz), p0, p1, p2);

            y.assign(max(y, select(range.greaterThanEqual(0), maxY.sub(range), y)));
          }
        );

        If(y.lessThan(minY), () => {
          Continue();
        });

        const shape = shapes.element(model.mul(2)).toVar();
        const flags = shapes.element(model.mul(2).add(1)).toVar();
        const least = shape.x.mul(0.5);
        const size = toRandomFloat(scale).mul(shape.y.mul(0.9).sub(least)).add(least).mul(uniforms.height).toVar();
        const turn = toRandomFloat(yaw)
          .mul(Math.PI * 2)
          .toVar();

        // `UpdateVisibleM`: a tuft's screen area by its slot's distance, against the engine's thresholds.
        const shrunk = size.mul(shrink).toVar();
        const area = shrunk.mul(shrunk).mul(shape.z).mul(shape.z).div(max(distance, 0.0001)).toVar();

        If(area.lessThan(discard), () => {
          Continue();
        });

        // The engine picks a waving tuft's wave by its own unseeded generator; this picks it by place, so it holds.
        const wave = select(
          int(x).add(z.mul(7)).add(sx.mul(3)).add(sz.mul(5)).mod(3).abs().equal(0),
          float(2),
          float(1)
        );
        const isWaving = flags.x.greaterThan(0.5).and(area.greaterThan(discard.mul(16)));
        const place = vec3(px, y, pz.negate()).toVar();

        If(toOutside(uniforms, place.add(vec3(0, shape.w.mul(shrunk).mul(0.5), 0)), shape.z.mul(shrunk)), () => {
          Continue();
        });

        const slot = (atomicAdd(counts.element(models), uint(1)) as unknown as Node<"uint">).toVar();

        If(slot.greaterThanEqual(uint(capacity)), () => {
          Continue();
        });

        items.element(slot.mul(GRASS_ITEM_VECTORS)).assign(vec4(place, turn));
        items.element(slot.mul(GRASS_ITEM_VECTORS).add(1)).assign(vec4(shrunk, hemi, sun, select(isWaving, wave, 0)));
        itemModels.element(slot).assign(model);
        atomicAdd(counts.element(model), uint(1));
      });
    });
  })().compute(Math.max(uniforms.slotCount, 1));
}

/** `CRandom::randI()`: the state advanced, and bits 16 to 30 of it. */
function toRandom(state: Node<"uint">): Node<"uint"> {
  state.assign(state.mul(uint(214013)).add(uint(2531011)));

  return bitAnd(shiftRight(state, uint(16)), uint(0x7fff)).toVar();
}

/** `CRandom::randF()`: `randI() / 32767`. */
function toRandomFloat(state: Node<"uint">): Node<"float"> {
  return float(toRandom(state)).div(32767);
}

/** `CRandom::randFs(range)`: `randF(-range, range)`. */
function toSignedRandom(state: Node<"uint">, range: Node<"float">): Node<"float"> {
  return toRandomFloat(state).mul(range.mul(2)).sub(range);
}

/**
 * `InterpolateAndDither`'s density: an object's four corner densities, each `255 * a / 15`, interpolated both ways
 * over the slot and averaged, as `Interpolate` does.
 */
function toDensity(palette: Node<"uint">, column: Node<"uint">, row: Node<"uint">, steps: Node<"int">): Node<"int"> {
  const [c0, c1, c2, c3] = [0, 4, 8, 12].map((bits: number) =>
    float(bitAnd(shiftRight(palette, uint(bits)), uint(0xf)))
      .mul(255)
      .div(15)
  );
  const fx = float(column).div(float(steps));
  const fy = float(row).div(float(steps));
  const c01 = c0.mul(fx.oneMinus()).add(c1.mul(fx));
  const c23 = c2.mul(fx.oneMinus()).add(c3.mul(fx));
  const c02 = c0.mul(fy.oneMinus()).add(c2.mul(fy));
  const c13 = c1.mul(fy.oneMinus()).add(c3.mul(fy));
  const interpolated = fy
    .oneMinus()
    .mul(c01)
    .add(fy.mul(c23))
    .add(fx.oneMinus().mul(c02).add(fx.mul(c13)))
    .div(2);

  const rounded = int(floor(interpolated.add(0.5)));

  return select(rounded.lessThan(0), int(0), select(rounded.greaterThan(255), int(255), rounded));
}

/**
 * `CDB::TestRayTri` straight down, culling back faces as the planting asks: how far below the origin the ray meets the
 * triangle, or minus one where it misses.
 */
function toRayRange(origin: Node<"vec3">, p0: Node<"vec3">, p1: Node<"vec3">, p2: Node<"vec3">): Node<"float"> {
  const direction = vec3(0, -1, 0);
  const edge1 = p1.sub(p0);
  const edge2 = p2.sub(p0);
  const pvec = cross(direction, edge2);
  const det = dot(edge1, pvec);
  const tvec = origin.sub(p0);
  const u = dot(tvec, pvec);
  const qvec = cross(tvec, edge1);
  const v = dot(direction, qvec);
  const isHit = det
    .greaterThanEqual(RAY_EPSILON)
    .and(u.greaterThanEqual(0))
    .and(u.lessThanEqual(det))
    .and(v.greaterThanEqual(0))
    .and(u.add(v).lessThanEqual(det));

  return select(isHit, dot(edge2, qvec).div(max(det, RAY_EPSILON)), float(-1));
}

/** Whether a sphere lies wholly outside one of the view's planes. */
function toOutside(uniforms: GrassUniforms, center: Node<"vec3">, radius: Node<"float">): Node<"bool"> {
  const first: Node<"vec4"> = uniforms.planeNodes.element(0) as unknown as Node<"vec4">;
  let outside: Node<"bool"> = dot(first.xyz, center).add(first.w).lessThan(radius.negate());

  for (let plane = 1; plane < 6; plane++) {
    const equation: Node<"vec4"> = uniforms.planeNodes.element(plane) as unknown as Node<"vec4">;

    outside = outside.or(dot(equation.xyz, center).add(equation.w).lessThan(radius.negate()));
  }

  return outside;
}
