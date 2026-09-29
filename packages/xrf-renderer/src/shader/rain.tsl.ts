import {
  cameraPosition,
  clamp,
  cos,
  cross,
  float,
  floor,
  fract,
  ivec2,
  mix,
  mod,
  normalize,
  select,
  shiftRight,
  sin,
  sqrt,
  textureLoad,
  uint,
  vec2,
  vec3,
} from "three/tsl";
import { Node } from "three/webgpu";

import { IRainFall } from "#/shader/rain-fall";
import { RainUniforms } from "#/uniforms/rain-uniforms";
import { RAIN_COVER_DEPTH, RAIN_COVER_RESOLUTION } from "#/visibility/rain-cover";

/** `source_offset`: metres above the camera a streak starts. */
const SOURCE_OFFSET: number = 40;

/** `max_distance`: metres a streak falls at most. */
const MAX_DISTANCE: number = SOURCE_OFFSET * 1.25;

/** `source_radius`: how far around the camera streaks start, as a square of the disc's area. */
const SOURCE_TILE: number = 12.5 * Math.sqrt(Math.PI);

/** `drop_speed_min` and `drop_speed_max`, in metres a second. */
const SPEED_MIN: number = 40;
const SPEED_MAX: number = 80;

/** `drop_angle`: how far a streak strays from the rain's way. */
const STRAY: number = (3 * Math.PI) / 180;

/** `drop_length`: metres a streak is long at its heaviest, `drop_width`: half its width. */
const STREAK_LENGTH: number = 5;
const STREAK_WIDTH: number = 0.3;

/** `particles_time`: seconds a splash lasts. */
export const RAIN_SPLASH_TIME: number = 0.3;

/**
 * A streak's fall, the engine's `Born` and `RenewItem` made stateless: each streak falls again and again at its own
 * speed, from a column of the level that the camera's square wraps as it moves, and stops where the cover under that
 * column stands, which is where its splash is.
 *
 * @param streak - Which streak, from nought.
 * @param rain - The rain's uniforms.
 * @returns Its fall now.
 */
export function toRainFall(streak: Node<"float">, rain: RainUniforms): IRainFall {
  const index: Node<"uint"> = uint(streak);

  function random(cycle: Node<"uint">, draw: number): Node<"float"> {
    return float(
      toHash(
        index
          .mul(uint(0x9e3779b1))
          .add(cycle.mul(uint(0x85ebca6b)))
          .add(uint(draw))
      )
    ).div(4294967295);
  }

  const speed: Node<"float"> = mix(float(SPEED_MIN), float(SPEED_MAX), random(uint(0), 0));
  const period: Node<"float"> = float(MAX_DISTANCE).div(speed);
  const phase: Node<"float"> = rain.time.div(period).add(random(uint(0), 1));
  const fall: Node<"uint"> = uint(floor(phase));
  const age: Node<"float"> = fract(phase).mul(period);

  function draw(at: number): Node<"float"> {
    return random(fall, at + 2);
  }

  // A column of the level, kept where it is as the camera moves and wrapped around it.
  const base: Node<"vec2"> = vec2(draw(0), draw(1)).mul(SOURCE_TILE);
  const around: Node<"vec2"> = mod(base.sub(cameraPosition.xz).add(SOURCE_TILE / 2), SOURCE_TILE).sub(SOURCE_TILE / 2);
  const column: Node<"vec3"> = vec3(cameraPosition.x.add(around.x), cameraPosition.y, cameraPosition.z.add(around.y));
  // `random_dir(axis, drop_angle)`: within the cone about the rain's way.
  const axis: Node<"vec3"> = rain.axis;
  const across: Node<"vec3"> = normalize(cross(axis, vec3(1, 0, 0)));
  const along: Node<"vec3"> = cross(axis, across);
  const lean: Node<"float"> = mix(float(1), float(Math.cos(STRAY)), draw(2));
  const turn: Node<"float"> = draw(3).mul(Math.PI * 2);
  const direction: Node<"vec3"> = normalize(
    axis.mul(lean).add(
      across
        .mul(cos(turn))
        .add(along.mul(sin(turn)))
        .mul(sqrt(float(1).sub(lean.mul(lean))))
    )
  );
  // It starts `source_offset` over the camera, set back along its way so it falls through its column.
  const start: Node<"float"> = float(SOURCE_OFFSET).div(direction.y.negate());
  const head: Node<"vec3"> = column.add(direction.mul(speed.mul(age).sub(start)));
  const cover: Node<"float"> = toRainCoverHeight(column, rain);
  const reach: Node<"float"> = cover.sub(column.y).div(direction.y);
  const landing: Node<"float"> = reach.add(start).div(speed);

  return {
    age,
    direction,
    head,
    isFalling: streak.lessThan(rain.count),
    landed: column.add(direction.mul(reach)),
    landing: select(landing.lessThan(period), landing, period.add(1)),
    random: draw,
  };
}

/**
 * @param fall - The streak's fall.
 * @param corner - The quad's corner: across the streak from minus one to one, then from its tail to its head.
 * @param rain - The rain's uniforms.
 * @returns Where the corner stands, the streak facing the camera from its tail to its head, or collapsed onto its head
 *   while it has landed or is not drawn.
 */
export function toRainStreakPosition(fall: IRainFall, corner: Node<"vec2">, rain: RainUniforms): Node<"vec3"> {
  const { head, direction } = fall;
  const tail: Node<"vec3"> = head.sub(direction.mul(rain.color.w.mul(STREAK_LENGTH)));
  const middle: Node<"vec3"> = mix(tail, head, 0.5);
  const side: Node<"vec3"> = cross(normalize(middle.sub(cameraPosition)), direction);
  const placed: Node<"vec3"> = mix(tail, head, corner.y).add(side.mul(corner.x.mul(STREAK_WIDTH)));

  return select(fall.isFalling.and(fall.age.lessThan(fall.landing)), placed, head);
}

/**
 * @param fall - The streak's fall.
 * @param corner - The quad's corner, as the streak takes it.
 * @returns Its texture coordinate: the engine's two sets, one mirroring the other, picked each fall.
 */
export function toRainStreakCoordinates(fall: IRainFall, corner: Node<"vec2">): Node<"vec2"> {
  const first: Node<"vec2"> = vec2(corner.y, float(1).sub(corner.x).mul(0.5));

  return select(fall.random(4).lessThan(0.5), first, vec2(1).sub(first));
}

/**
 * `CEffect_Rain::Hit`: half the drops that land leave a splash, turned at random and shrinking to nothing.
 *
 * @param fall - The streak's fall.
 * @param vertex - A vertex of the splash's model, in renderer space.
 * @returns Where it stands while the splash lasts, collapsed onto where it landed otherwise.
 */
export function toRainSplashPosition(fall: IRainFall, vertex: Node<"vec3">): Node<"vec3"> {
  const since: Node<"float"> = fall.age.sub(fall.landing);
  const scale: Node<"float"> = float(1).sub(since.div(RAIN_SPLASH_TIME));
  const turn: Node<"float"> = fall.random(5).mul(Math.PI * 2);
  const c: Node<"float"> = cos(turn);
  const s: Node<"float"> = sin(turn);
  const turned: Node<"vec3"> = vec3(
    vertex.x.mul(c).add(vertex.z.mul(s)),
    vertex.y,
    vertex.z.mul(c).sub(vertex.x.mul(s))
  );
  const isShown: Node<"bool"> = fall.isFalling
    .and(fall.random(6).lessThan(0.5))
    .and(since.greaterThanEqual(0))
    .and(since.lessThan(RAIN_SPLASH_TIME));

  return select(isShown, fall.landed.add(turned.mul(scale)), fall.landed);
}

/**
 * The height of the first thing over a column that a drop lands on: read from the cover's depth, straight down from
 * the height it is seen from; nothing at all outside it.
 *
 * @param column - A point of the column, in world space.
 * @param rain - The rain's uniforms, the cover among them.
 * @returns The height, in world space.
 */
export function toRainCoverHeight(column: Node<"vec3">, rain: RainUniforms): Node<"float"> {
  const { window } = rain;
  const uv: Node<"vec2"> = vec2(column.x.sub(window.x), column.z.sub(window.y)).div(window.z.mul(2)).add(0.5);
  const isInside: Node<"bool"> = uv.x
    .greaterThanEqual(0)
    .and(uv.x.lessThan(1))
    .and(uv.y.greaterThanEqual(0))
    .and(uv.y.lessThan(1));
  const texel: Node<"vec2"> = clamp(uv.mul(RAIN_COVER_RESOLUTION), vec2(0), vec2(RAIN_COVER_RESOLUTION - 1));
  // Reversed: one at the height it is seen from, nought where nothing stands.
  const depth: Node<"float"> = textureLoad(rain.coverDepth, ivec2(texel)).x;

  return select(isInside, window.w.sub(float(1).sub(depth).mul(RAIN_COVER_DEPTH)), float(-1e9));
}

/** `pcg`: a word hashed into another, every bit of it depending on every bit given. */
function toHash(seed: Node<"uint">): Node<"uint"> {
  const state: Node<"uint"> = seed.mul(uint(747796405)).add(uint(2891336453));
  const word: Node<"uint"> = shiftRight(state, shiftRight(state, uint(28)).add(uint(4)))
    .bitXor(state)
    .mul(uint(277803737));

  return shiftRight(word, uint(22)).bitXor(word);
}
