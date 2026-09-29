import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { EngineUniforms } from "#/uniforms/engine-uniforms";
import { RainUniforms } from "#/uniforms/rain-uniforms";
import { WetUniforms } from "#/uniforms/wet-uniforms";

/**
 * What `rain_patch_normal` reads: the G-buffer, the camera it was drawn from, the rain's cover, the wet surfaces and
 * the engine they are wet as.
 */
export interface IWetPatchInput {
  textures: IGBufferTextures;
  camera: CameraUniforms;
  rain: RainUniforms;
  wet: WetUniforms;
  engine: EngineUniforms;
}
