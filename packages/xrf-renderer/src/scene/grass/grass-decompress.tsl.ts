import {
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
import { ComputeNode, Node } from "three/webgpu";

import { RENDERER_GRASS_SLOT_WORDS, RENDERER_GRASS_TRIANGLE_FLOATS } from "#/contract/scene/renderer-grass";
import { IGrassBuffers } from "#/scene/grass/grass-buffers";
import { GRASS_CACHE_KEY_WORDS, GRASS_CACHE_VECTORS } from "#/scene/grass/grass-cache-buffers";
import { toGrassCacheItems, toGrassCacheKeys, toGrassCacheShapes } from "#/scene/grass/grass-cache.tsl";
import { toGrassRandom, toGrassRandomFloat, toGrassSignedRandom } from "#/scene/grass/grass-random.tsl";
import { IGrassRingSlot } from "#/scene/grass/grass-ring-slot";
import { isGrassCellCurrent, toGrassRingCells, toGrassRingSlot } from "#/scene/grass/grass-ring.tsl";
import { isGrassCellAdmitted } from "#/scene/grass/grass-schedule.tsl";
import { GRASS_BOX_GROWTH } from "#/scene/grass/grass-view.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/** The seed `cache_Decompress` starts every one of a slot's four generators from, before its slot is mixed in. */
const SEED: number = 0x12071980;

/** `DetailSlot::ID_Empty`: a corner planting nothing. */
const EMPTY_ID: number = 0x3f;

/** `EPS`: the determinant below which a ray is taken to lie in a triangle's plane (`xrCore/math_constants.h`). */
const RAY_EPSILON: number = 0.00001;

/**
 * `cache_Decompress`, a thread a cell of the ring: a stale cell the schedule admits plants the world slot it holds now,
 * with the same generators seeded the same way and drawn in the same order, the same dither, the same ray cast down
 * onto the slot's own triangles, so a slot plants what the engine plants in it. What a tuft keeps whatever the camera
 * does is kept: its place, turn and size, its model and its wave, the slot's height and light beside.
 *
 * @param buffers - What the grass is planted from and into.
 * @param uniforms - Where the camera stands and what the planting is set to.
 * @param ring - How far the ring reaches.
 * @returns The pass; its dispatch is the ring's cells, set before each frame's.
 */
export function createGrassDecompress(
  buffers: IGrassBuffers,
  uniforms: GrassUniforms,
  ring: GrassRingUniforms
): ComputeNode {
  const { level, cache } = buffers;
  const models: number = level.modelCount;
  const { perCell } = cache;
  const grid = storage(level.grid, "uint", level.gridLength).toReadOnly();
  const slots = storage(level.slots, "uint", level.slotWords).toReadOnly();
  const bins = storage(level.bins, "uint", level.binLength).toReadOnly();
  const triangles = storage(level.triangles, "float", level.triangleFloats).toReadOnly();
  const dither = storage(level.dither, "uint", 256).toReadOnly();
  const shapes = storage(level.models, "vec4", Math.max(models, 1) * 2).toReadOnly();
  const keys = toGrassCacheKeys(cache);
  const cellShapes = toGrassCacheShapes(cache);
  const cached = toGrassCacheItems(cache);

  return Fn(() => {
    const cell = int(instanceIndex).toVar();

    If(cell.greaterThanEqual(toGrassRingCells(ring)), () => {
      Return();
    });

    const slot: IGrassRingSlot = toGrassRingSlot(uniforms, ring, cell);
    const generation = uint(uniforms.generation).toVar();

    If(isGrassCellCurrent(keys, cell, slot, generation), () => {
      Return();
    });

    // Built after the return above: asking takes a ticket, which only a stale cell may spend.
    If(isGrassCellAdmitted(cache, slot.band).not(), () => {
      Return();
    });

    const sx = slot.x;
    const sz = slot.z;
    const key = uint(cell).mul(GRASS_CACHE_KEY_WORDS).toVar();

    keys.element(key).assign(uint(sx));
    keys.element(key.add(1)).assign(uint(sz));
    keys.element(key.add(2)).assign(generation);
    keys.element(key.add(3)).assign(uint(0));

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
    const sun = float(bitAnd(shiftRight(w1, uint(12)), uint(0xf))).div(15);
    const hemi = float(bitAnd(shiftRight(w1, uint(16)), uint(0xf))).div(15);
    const palettes: ReadonlyArray<Node<"uint">> = [
      bitAnd(w2, uint(0xffff)),
      shiftRight(w2, uint(16)),
      bitAnd(w3, uint(0xffff)),
      shiftRight(w3, uint(16)),
    ].map((palette) => palette.toVar());

    cellShapes.element(cell).assign(vec4(base.add(top).mul(0.5), top.sub(base).mul(0.5), hemi, sun));

    // `vis.box`, grown by `EPS_L`.
    const minX = float(sx).mul(2).sub(GRASS_BOX_GROWTH).toVar();
    const minZ = float(sz).mul(2).sub(GRASS_BOX_GROWTH).toVar();
    const minY = base.sub(GRASS_BOX_GROWTH).toVar();
    const maxY = top.add(GRASS_BOX_GROWTH).toVar();

    const seed = uint(SEED)
      .bitXor(uint(sx.mul(sz)))
      .toVar();
    const selection = seed.toVar();
    const jitter = seed.toVar();
    const yaw = seed.toVar();
    const scale = seed.toVar();
    const steps = int(uniforms.steps).toVar();
    const first = uint(cell).mul(perCell).toVar();
    const held = uint(0).toVar();

    // Each loop names its own counter: three names every loop's `i`, so a nested one would shadow the one outside it.
    loopNamed({ condition: "<=", end: steps, name: "row", start: int(0), type: "int" }, (z: Node<"int">) => {
      loopNamed({ condition: "<=", end: steps, name: "column", start: int(0), type: "int" }, (x: Node<"int">) => {
        const shiftX = toGrassRandom(jitter).mod(16).toVar();
        const shiftZ = toGrassRandom(jitter).mod(16).toVar();
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
          pick.assign(toGrassRandom(selection).mod(selected));
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
        const jitterZ = toGrassSignedRandom(jitter, uniforms.jitter).toVar();
        const jitterX = toGrassSignedRandom(jitter, uniforms.jitter).toVar();
        const px = float(x).div(float(steps)).mul(2).add(minX).add(jitterX).toVar();
        const pz = float(z).div(float(steps)).mul(2).add(minZ).add(jitterZ).toVar();
        const y = minY.sub(5).toVar();
        // The slot is walked in engine's space and its triangles arrive in the renderer's, whose z runs the other way.
        const origin = vec3(px, maxY, pz.negate()).toVar();

        loopNamed(
          { end: binStart.add(binCount), name: "entry", start: binStart, type: "uint" },
          (entry: Node<"uint">) => {
            const corners = bins.element(entry).mul(RENDERER_GRASS_TRIANGLE_FLOATS).toVar();
            const [p0, p1, p2] = [0, 1, 2].map((corner: number) =>
              vec3(
                triangles.element(corners.add(corner * 3)),
                triangles.element(corners.add(corner * 3 + 1)),
                triangles.element(corners.add(corner * 3 + 2))
              )
            );
            const range = toRayRange(origin, p0, p1, p2);

            y.assign(max(y, select(range.greaterThanEqual(0), maxY.sub(range), y)));
          }
        );

        If(y.lessThan(minY), () => {
          Continue();
        });

        const shape = shapes.element(model.mul(2)).toVar();
        const least = shape.x.mul(0.5);
        // Before the settings' height scales it, which a frame applies, so a height changed plants nothing again.
        const size = toGrassRandomFloat(scale).mul(shape.y.mul(0.9).sub(least)).add(least).toVar();
        const turn = toGrassRandomFloat(yaw)
          .mul(Math.PI * 2)
          .toVar();
        // The engine picks a waving tuft's wave by its own unseeded generator; this picks it by place, so it holds.
        const wave = select(
          int(x).add(z.mul(7)).add(sx.mul(3)).add(sz.mul(5)).mod(3).abs().equal(0),
          float(2),
          float(1)
        );

        If(held.lessThan(uint(perCell)), () => {
          const to = first.add(held).mul(GRASS_CACHE_VECTORS);

          cached.element(to).assign(vec4(px, y, pz.negate(), turn));
          cached.element(to.add(1)).assign(vec4(size, float(model), wave, 0));
          held.addAssign(1);
        });
      });
    });

    keys.element(key.add(3)).assign(held);
  })().compute(cache.cells);
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
