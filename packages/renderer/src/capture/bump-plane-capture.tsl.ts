import { screenUV, vec3, vec4 } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { ERendererBumpPlane } from "#/contract/renderer-capture";
import { MaterialSamplers } from "#/material/material-samplers";
import { decodeBumpGloss, decodeBumpHeight, decodeBumpNormal } from "#/shader/bump.tsl";
import { getFlatBumpCompanionTexture, getFlatBumpTexture } from "#/texture/placeholder-textures";

/**
 * @param plane - The plane wanted.
 * @param samplers - Where the pair is bound.
 * @param bump - The key of the pair's first half.
 * @param companion - The key of its companion.
 * @returns What the plane shows at each texel, through the same decode the surfaces shade with.
 */
export function toBumpPlaneFragment(
  plane: ERendererBumpPlane,
  samplers: MaterialSamplers,
  bump: string,
  companion: string
): Node<"vec4"> {
  return vec4(
    toBumpPlaneColor(
      plane,
      samplers.bind(bump, getFlatBumpTexture(), screenUV),
      samplers.bind(companion, getFlatBumpCompanionTexture(), screenUV)
    ),
    1
  );
}

function toBumpPlaneColor(plane: ERendererBumpPlane, bump: TextureNode, companion: TextureNode): Node<"vec3"> {
  switch (plane) {
    case ERendererBumpPlane.BUMP:
      return bump.xyz;

    case ERendererBumpPlane.COMPANION:
      return companion.xyz;

    case ERendererBumpPlane.NORMAL:
      return decodeBumpNormal(bump, companion).mul(0.5).add(0.5);

    case ERendererBumpPlane.GLOSS:
      return vec3(decodeBumpGloss(bump));

    case ERendererBumpPlane.HEIGHT:
      return vec3(decodeBumpHeight(companion));
  }
}
