import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { MotionUniforms } from "#/uniforms/motion-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/** The uniforms the resolve reads. */
export interface ITemporalResolveUniforms {
  camera: CameraUniforms;
  motion: MotionUniforms;
  temporal: TemporalUniforms;
}
