// @ts-nocheck
// Ported from three.js r186, `examples/jsm/tsl/display/SMAANode.js` (MIT, Copyright 2010-2025 three.js authors),
// itself SMAA 1x Medium with colour edge detection, v2.8 (MIT, Jorge Jimenez, Jose I. Echevarria, Belen Masia,
// Fernando Navarro, Diego Gutierrez). The stages are three's own, word for word but for what they read: the node
// cannot be used as it ships, since it loads its lookup textures through `Image`, which a worker does not have. TSL
// assigns through swizzles (`a.xz = ...`), which its typings cannot express, so the stages are left untyped.

import { abs, Break, dot, float, Fn, If, int, Loop, max, mix, sign, sqrt, step, uv, vec2, vec4 } from "three/tsl";
import { Node, TextureNode, UniformNode, Vector2 } from "three/webgpu";

/** What the three stages read. */
export interface ISmaaInputs {
  /** The frame being smoothed. */
  sourceTexture: TextureNode;
  /** What the first stage wrote. */
  edgesTexture: TextureNode;
  /** What the second stage wrote. */
  weightsTexture: TextureNode;
  /** `AreaTex`, the precomputed blending areas. */
  areaTexture: TextureNode;
  /** `SearchTex`, the precomputed search lengths. */
  searchTexture: TextureNode;
  /** One over the frame's size in pixels. */
  invSize: UniformNode<"vec2", Vector2>;
}

/** Each stage's fragment, in the order they run. */
export interface ISmaaStages {
  edges: Node<"vec4">;
  weights: Node<"vec4">;
  blend: Node<"vec4">;
}

/**
 * @param inputs - What the stages read.
 * @returns SMAA's three stages: edge detection, blending weights, and neighbourhood blending.
 */
export function toSmaaStages(inputs: ISmaaInputs): ISmaaStages {
  const { sourceTexture, edgesTexture, weightsTexture, areaTexture, searchTexture, invSize } = inputs;

  const SMAA_THRESHOLD = 0.1;
  const SMAA_MAX_SEARCH_STEPS = 8;
  const SMAA_AREATEX_MAX_DISTANCE = 16;
  const SMAA_AREATEX_PIXEL_SIZE = vec2(1 / 160, 1 / 560);
  const SMAA_AREATEX_SUBTEX_SIZE = 1 / 7;

  const uvNode = uv();

  // edges

  const SMAAEdgeDetection = Fn(() => {
    const vOffset0 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(-1.0, 0.0, 0.0, -1.0)))
      .toVertexStage();
    const vOffset1 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(1.0, 0.0, 0.0, 1.0)))
      .toVertexStage();
    const vOffset2 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(-2.0, 0.0, 0.0, -2.0)))
      .toVertexStage();

    const threshold = vec2(SMAA_THRESHOLD, SMAA_THRESHOLD);

    // Calculate color deltas:
    const delta = vec4().toVar();
    const C = sourceTexture.sample(uvNode).rgb.toVar();

    // Calculate left and top deltas:
    const Cleft = sourceTexture.sample(vOffset0.xy).rgb.toVar();
    let t = abs(C.sub(Cleft));

    delta.x = max(t.r, t.g, t.b);

    const Ctop = sourceTexture.sample(vOffset0.zw).rgb.toVar();

    t = abs(C.sub(Ctop));
    delta.y = max(t.r, t.g, t.b);

    // We do the usual threshold:
    const edges = step(threshold, delta.xy).toVar();

    // Then discard if there is no edge:
    dot(edges, vec2(1.0, 1.0)).equal(0).discard();

    // Calculate right and bottom deltas:
    const Cright = sourceTexture.sample(vOffset1.xy).rgb.toVar();

    t = abs(C.sub(Cright));
    delta.z = max(t.r, t.g, t.b);

    const Cbottom = sourceTexture.sample(vOffset1.zw).rgb.toVar();

    t = abs(C.sub(Cbottom));
    delta.w = max(t.r, t.g, t.b);

    // Calculate the maximum delta in the direct neighborhood:
    let maxDelta = max(delta.x, delta.y, delta.z, delta.w).toVar();

    // Calculate left-left and top-top deltas:
    const Cleftleft = sourceTexture.sample(vOffset2.xy).rgb.toVar();

    t = abs(C.sub(Cleftleft));
    delta.z = max(t.r, t.g, t.b);

    const Ctoptop = sourceTexture.sample(vOffset2.zw).rgb.toVar();

    t = abs(C.sub(Ctoptop));
    delta.w = max(t.r, t.g, t.b);

    // Calculate the final maximum delta:
    maxDelta = max(maxDelta, delta.z, delta.w);

    // Local contrast adaptation in action:
    edges.xy.mulAssign(vec2(step(float(0.5).mul(maxDelta), delta.xy)));

    return vec4(edges, 0, 0);
  });

  // weights

  const SMAASearchLength = Fn(([searchTex, e, bias, scale]) => {
    // Not required if searchTex accesses are set to point:
    // float2 SEARCH_TEX_PIXEL_SIZE = 1.0 / float2(66.0, 33.0);
    // e = float2(bias, 0.0) + 0.5 * SEARCH_TEX_PIXEL_SIZE + e * float2(scale, 1.0) * float2(64.0, 32.0) * SEARCH_TEX_PIXEL_SIZE;
    const coord = vec2(e).toVar();

    coord.r = bias.add(coord.r.mul(scale));

    return float(255).mul(searchTex.sample(coord)).r;
  });

  const SMAAArea = Fn(([areaTex, dist, e1, e2, offset]) => {
    // Rounding prevents precision errors of bilinear filtering:
    let texcoord = float(SMAA_AREATEX_MAX_DISTANCE)
      .mul(float(4).mul(vec2(e1, e2)).round())
      .add(dist);

    // We do a scale and bias for mapping to texel space:
    texcoord = SMAA_AREATEX_PIXEL_SIZE.mul(texcoord).add(float(0.5).mul(SMAA_AREATEX_PIXEL_SIZE));

    // Move to proper place, according to the subpixel offset:
    texcoord.y.addAssign(float(SMAA_AREATEX_SUBTEX_SIZE).mul(offset));

    return areaTex.sample(texcoord).rg;
  });

  const SMAASearchXLeft = Fn(([edgesTex, searchTex, texcoord, end]) => {
    /**
     * @PSEUDO_GATHER4
     * This texcoord has been offset by (-0.25, -0.125) in the vertex shader to
     * sample between edge, thus fetching four edges in a row.
     * Sampling with different offsets in each direction allows to disambiguate
     * which edges are active from the four fetched ones.
     */

    const e = vec2(0.0, 1.0).toVar();
    const coord = vec2(texcoord).toVar();

    Loop({ start: int(0), end: int(SMAA_MAX_SEARCH_STEPS), type: "int", condition: "<" }, () => {
      // port note: Changed while to for

      e.assign(edgesTex.sample(coord).rg);
      coord.subAssign(vec2(2, 0).mul(invSize));

      If(coord.x.lessThanEqual(end).or(e.g.lessThanEqual(float(0.8281)).or(e.r.notEqual(float(0)))), () => {
        Break();
      });
    });

    // We correct the previous (-0.25, -0.125) offset we applied:
    coord.x.addAssign(float(0.25).mul(invSize.x));

    // The searches are bias by 1, so adjust the coords accordingly:
    coord.x.addAssign(invSize.x);

    // Disambiguate the length added by the last step:
    coord.x.addAssign(float(2).mul(invSize.x));
    coord.x.subAssign(invSize.x.mul(SMAASearchLength(searchTex, e, 0, 0.5)));

    return coord.x;
  });

  const SMAASearchXRight = Fn(([edgesTex, searchTex, texcoord, end]) => {
    const e = vec2(0.0, 1.0).toVar();
    const coord = vec2(texcoord).toVar();

    Loop({ start: int(0), end: int(SMAA_MAX_SEARCH_STEPS), type: "int", condition: "<" }, () => {
      // port note: Changed while to for

      e.assign(edgesTex.sample(coord).rg);
      coord.addAssign(vec2(2, 0).mul(invSize));

      If(coord.x.greaterThanEqual(end).or(e.g.lessThanEqual(float(0.8281)).or(e.r.notEqual(float(0)))), () => {
        Break();
      });
    });

    coord.x.subAssign(float(0.25).mul(invSize.x));
    coord.x.subAssign(invSize.x);
    coord.x.subAssign(float(2).mul(invSize.x));
    coord.x.addAssign(invSize.x.mul(SMAASearchLength(searchTex, e, 0.5, 0.5)));

    return coord.x;
  });

  const SMAASearchYUp = Fn(([edgesTex, searchTex, texcoord, end]) => {
    const e = vec2(1.0, 0.0).toVar();
    const coord = vec2(texcoord).toVar();

    Loop({ start: int(0), end: int(SMAA_MAX_SEARCH_STEPS), type: "int", condition: "<" }, () => {
      // port note: Changed while to for

      e.assign(edgesTex.sample(coord).rg);
      coord.addAssign(vec2(0, -2).mul(invSize));

      If(coord.y.lessThanEqual(end).or(e.r.lessThanEqual(float(0.8281)).or(e.g.notEqual(float(0)))), () => {
        Break();
      });
    });

    coord.y.addAssign(float(0.25).mul(invSize.y));
    coord.y.addAssign(invSize.y);
    coord.y.addAssign(float(2).mul(invSize.y));
    coord.y.subAssign(invSize.y.mul(SMAASearchLength(searchTex, e.gr, 0, 0.5)));

    return coord.y;
  });

  const SMAASearchYDown = Fn(([edgesTex, searchTex, texcoord, end]) => {
    const e = vec2(1.0, 0.0).toVar();
    const coord = vec2(texcoord).toVar();

    Loop({ start: int(0), end: int(SMAA_MAX_SEARCH_STEPS), type: "int", condition: "<" }, () => {
      // port note: Changed while to for

      e.assign(edgesTex.sample(coord).rg);
      coord.subAssign(vec2(0, -2).mul(invSize));

      If(coord.y.greaterThanEqual(end).or(e.r.lessThanEqual(float(0.8281)).or(e.g.notEqual(float(0)))), () => {
        Break();
      });
    });

    coord.y.subAssign(float(0.25).mul(invSize.y));
    coord.y.subAssign(invSize.y);
    coord.y.subAssign(float(2).mul(invSize.y));
    coord.y.addAssign(invSize.y.mul(SMAASearchLength(searchTex, e.gr, 0.5, 0.5)));

    return coord.y;
  });

  const SMAAWeights = Fn(() => {
    const vPixcoord = uvNode.xy.div(invSize).toVertexStage();

    // We will use these offsets for the searches later on (see @PSEUDO_GATHER4):
    const vOffset0 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(-0.25, -0.125, 1.25, -0.125)))
      .toVertexStage();
    const vOffset1 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(-0.125, -0.25, -0.125, 1.25)))
      .toVertexStage();

    // And these for the searches, they indicate the ends of the loops:
    const vOffset2 = vec4(vOffset0.xz, vOffset1.yw)
      .add(vec4(-2.0, 2.0, -2.0, 2.0).mul(vec4(invSize.xx, invSize.yy)).mul(float(SMAA_MAX_SEARCH_STEPS)))
      .toVertexStage();

    const weights = vec4(0.0, 0.0, 0.0, 0.0).toVar();
    const subsampleIndices = vec4(0.0, 0.0, 0.0, 0.0).toVar();

    const e = edgesTexture.sample(uvNode).rg.toVar();

    If(e.g.greaterThan(float(0)), () => {
      // Edge at north

      let d = vec2().toVar();

      // Find the distance to the left:

      const coordsLeft = vec2().toVar();

      coordsLeft.x = SMAASearchXLeft(edgesTexture, searchTexture, vOffset0.xy, vOffset2.x);
      coordsLeft.y = vOffset1.y; // offset[1].y = texcoord.y - 0.25 * resolution.y (@CROSSING_OFFSET)
      d.x = coordsLeft.x;

      // Now fetch the left crossing edges, two at a time using bilinear
      // filtering. Sampling at -0.25 (see @CROSSING_OFFSET) enables to
      // discern what value each edge has:
      const e1 = edgesTexture.sample(coordsLeft).r.toVar();

      // Find the distance to the right:
      const coordsRight = vec2().toVar();

      coordsRight.x = SMAASearchXRight(edgesTexture, searchTexture, vOffset0.zw, vOffset2.y);
      coordsRight.y = vOffset1.y;
      d.y = coordsRight.x;

      // We want the distances to be in pixel units (doing this here allow to
      // better interleave arithmetic and memory accesses):
      d = d.div(invSize.x).sub(vPixcoord.x);

      // SMAAArea below needs a sqrt, as the areas texture is compressed quadratically:
      const sqrt_d = sqrt(abs(d));

      // Fetch the right crossing edges:
      const e2 = edgesTexture.sample(coordsRight.add(vec2(1, 0).mul(invSize))).r.toVar();

      weights.r = e2;

      // Get the area for this direction:
      weights.rg = SMAAArea(areaTexture, sqrt_d, e1, e2, float(subsampleIndices.y));
    });

    If(e.r.greaterThan(float(0)), () => {
      // Edge at west

      let d = vec2().toVar();

      // Find the distance to the top:

      const coordsUp = vec2().toVar();

      coordsUp.y = SMAASearchYUp(edgesTexture, searchTexture, vOffset1.xy, vOffset2.z);
      coordsUp.x = vOffset0.x; // offset[1].x = texcoord.x - 0.25 * resolution.x;
      d.x = coordsUp.y;

      // Fetch the top crossing edges:
      const e1 = edgesTexture.sample(coordsUp).g.toVar();

      // Find the distance to the bottom:
      const coordsDown = vec2().toVar();

      coordsDown.y = SMAASearchYDown(edgesTexture, searchTexture, vOffset1.zw, vOffset2.w);
      coordsDown.x = vOffset0.x;
      d.y = coordsDown.y;

      // We want the distances to be in pixel units:
      d = d.div(invSize.y).sub(vPixcoord.y);

      // SMAAArea below needs a sqrt, as the areas texture is compressed quadratically:
      const sqrt_d = sqrt(abs(d));

      // Fetch the bottom crossing edges:
      const e2 = edgesTexture.sample(coordsDown.add(vec2(0, 1).mul(invSize))).g.toVar();

      // Get the area for this direction:
      weights.ba = SMAAArea(areaTexture, sqrt_d, e1, e2, float(subsampleIndices.x));
    });

    return weights;
  });

  // blend

  const SMAABlend = Fn(() => {
    const vOffset1 = vec4(uvNode.xy, uvNode.xy)
      .add(vec4(invSize.xy, invSize.xy).mul(vec4(1.0, 0.0, 0.0, 1.0)))
      .toVertexStage();
    const result = vec4().toVar();

    // Fetch the blending weights for current pixel:

    const a = vec4().toVar();

    a.xz = weightsTexture.sample(uvNode).xz;
    a.y = weightsTexture.sample(vOffset1.zw).g;
    a.w = weightsTexture.sample(vOffset1.xy).a;

    // Is there any blending weight with a value greater than 0.0?

    If(dot(a, vec4(1.0)).lessThan(1e-5), () => {
      // Edge at north

      result.assign(sourceTexture.sample(uvNode));
    }).Else(() => {
      // Up to 4 lines can be crossing a pixel (one through each edge). We
      // favor blending by choosing the line with the maximum weight for each
      // direction:

      const offset = vec2().toVar();

      offset.x = a.a.greaterThan(a.b).select(a.a, a.b.negate()); // left vs. right
      offset.y = a.g.greaterThan(a.r).select(a.g, a.r.negate()); // top vs. bottom

      // Then we go in the direction that has the maximum weight:

      If(abs(offset.x).greaterThan(abs(offset.y)), () => {
        // horizontal vs. vertical

        offset.y.assign(0);
      }).Else(() => {
        offset.x.assign(0);
      });

      // Fetch the opposite color and lerp by hand:

      const C = sourceTexture.sample(uvNode).toVar();
      const texcoord = vec2(uvNode).toVar();

      texcoord.addAssign(sign(offset).mul(invSize));

      const Cop = sourceTexture.sample(texcoord).toVar();
      const s = abs(offset.x).greaterThan(abs(offset.y)).select(abs(offset.x), abs(offset.y)).toVar();

      const mixed = mix(C, Cop, s);

      result.assign(mixed);
    });

    return result;
  });

  return { blend: SMAABlend(), edges: SMAAEdgeDetection(), weights: SMAAWeights() };
}
