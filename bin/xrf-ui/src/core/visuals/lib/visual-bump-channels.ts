import { DoubleSide, IUniform, ShaderMaterial } from "three";

import {
  IVisualBumpTextures,
  XRAY_BUMP_GLOSS_GLSL,
  XRAY_BUMP_HEIGHT_GLSL,
  XRAY_BUMP_NORMAL_GLSL,
} from "@/core/visuals/lib/visual-bump";

/**
 * One plane of a bump pair, as stored or as the engine reconstructs it.
 *
 * The two raw views are the files themselves, which is what makes the other three checkable: a normal that looks wrong
 * is either a wrong plane or a wrong decode, and only seeing both answers which.
 */
export enum EVisualBumpView {
  /** `normal.gloss` as uploaded: the normal reversed into green, blue and alpha, gloss in red. */
  BUMP = "bump",
  /** `normal_error.height` as uploaded: the quantisation error of the three normal channels in rgb, height in alpha. */
  COMPANION = "companion",
  /** The tangent-space normal the engine reconstructs, mapped into the unit range to be looked at. */
  NORMAL = "normal",
  /** Gloss, which the engine feeds to a specular power. */
  GLOSS = "gloss",
  /** Height as the file stores it, which the parallax path samples and the deferred loader does not. */
  HEIGHT = "height",
}

/** A material drawing one view at a time, and the switch between them. */
export interface IVisualBumpChannels {
  material: ShaderMaterial;
  setView(view: EVisualBumpView): void;
  dispose(): void;
}

/** What the shader's view uniform holds for each view, which is the only place the two vocabularies meet. */
const VIEW_INDEX: Record<EVisualBumpView, number> = {
  [EVisualBumpView.BUMP]: 0,
  [EVisualBumpView.COMPANION]: 1,
  [EVisualBumpView.NORMAL]: 2,
  [EVisualBumpView.GLOSS]: 3,
  [EVisualBumpView.HEIGHT]: 4,
};

const VERTEX_SHADER: string = `
varying vec2 vXrayUv;

void main() {
  vXrayUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}
`;

/**
 * The decode of `gl/sload.h` again, this time with nothing else in the way.
 *
 * The three reconstructions are the same expressions the lit patch substitutes, imported rather than retyped: a second
 * spelling of the decode is exactly the thing these views exist to catch, so it must not be possible to have one.
 *
 * Nothing here is colour. The values are written as they are read, with no output encoding, because a plane of packed
 * numbers shown through a colour transform is no longer the plane.
 */
const FRAGMENT_SHADER: string = `
uniform sampler2D xrayBump;
uniform sampler2D xrayBumpX;
uniform int xrayBumpView;
varying vec2 vXrayUv;

void main() {
  // Sampled bottom row first, because X-Ray stores rows top first and neither texture loader flips one: a tile drawn
  // straight from three.js' generated uvs shows every plane upside down against the picture of the same file.
  vec2 xrayUv = vec2( vXrayUv.x, 1.0 - vXrayUv.y );
  vec4 xrayNu = texture2D( xrayBump, xrayUv );
  vec4 xrayNuE = texture2D( xrayBumpX, xrayUv );
  vec3 xrayChannel;

  if ( xrayBumpView == ${VIEW_INDEX[EVisualBumpView.BUMP]} ) {
    xrayChannel = xrayNu.rgb;
  } else if ( xrayBumpView == ${VIEW_INDEX[EVisualBumpView.COMPANION]} ) {
    xrayChannel = xrayNuE.rgb;
  } else if ( xrayBumpView == ${VIEW_INDEX[EVisualBumpView.NORMAL]} ) {
    xrayChannel = ( ${XRAY_BUMP_NORMAL_GLSL} ) * 0.5 + 0.5;
  } else if ( xrayBumpView == ${VIEW_INDEX[EVisualBumpView.GLOSS]} ) {
    xrayChannel = vec3( ${XRAY_BUMP_GLOSS_GLSL} );
  } else {
    xrayChannel = vec3( ${XRAY_BUMP_HEIGHT_GLSL} );
  }

  gl_FragColor = vec4( xrayChannel, 1.0 );
}
`;

/**
 * Builds the material the channel views are drawn with.
 *
 * A `ShaderMaterial` rather than a patch, because nothing about these views is lit: they answer what the two files
 * hold and what the engine makes of them, and a light shining on that answer would only be in the way.
 *
 * @param textures - The uploaded pair to sample.
 * @returns The material, its view switch, and the release of what it owns.
 */
export function createXrayBumpChannels(textures: IVisualBumpTextures): IVisualBumpChannels {
  const view: IUniform<number> = { value: VIEW_INDEX[EVisualBumpView.BUMP] };

  const material: ShaderMaterial = new ShaderMaterial({
    fragmentShader: FRAGMENT_SHADER,
    side: DoubleSide,
    uniforms: {
      xrayBump: { value: textures.bump },
      xrayBumpView: view,
      xrayBumpX: { value: textures.companion },
    },
    vertexShader: VERTEX_SHADER,
  });

  return {
    material,
    setView(next: EVisualBumpView): void {
      view.value = VIEW_INDEX[next];
    },
    dispose(): void {
      material.dispose();
    },
  };
}
