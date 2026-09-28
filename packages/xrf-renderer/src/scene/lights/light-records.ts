import { StorageBufferAttribute, Vector3 } from "three/webgpu";

import { queueBufferUpload } from "#/scene/buffer-upload";
import { LIGHT_RECORD, LIGHT_VECTORS, MAX_LIGHTS } from "#/scene/lights/light-record";
import { TLightRecordVector } from "#/scene/lights/light-record-vector";

/**
 * The records of the lights standing in view this frame, `LIGHT_VECTORS` vectors of four floats each, written in
 * place and uploaded as far as the frame wrote them.
 */
export class LightRecords {
  public readonly buffer: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(MAX_LIGHTS * LIGHT_VECTORS * 4),
    4
  );

  private readonly data: Float32Array = this.buffer.array as Float32Array;

  /**
   * @param light - The light's place among the records.
   * @param vector - Which of its vectors.
   * @param x - The vector's first float.
   * @param y - Its second.
   * @param z - Its third.
   * @param w - Its fourth.
   */
  public set(light: number, vector: TLightRecordVector, x: number, y: number, z: number, w: number): void {
    this.write((light * LIGHT_VECTORS + vector) * 4, x, y, z, w);
  }

  /**
   * @param light - The light's place among the records.
   * @param vector - Which of its vectors.
   * @param xyz - Its first three floats.
   * @param w - Its fourth.
   */
  public setVector(light: number, vector: TLightRecordVector, xyz: Vector3, w: number): void {
    this.set(light, vector, xyz.x, xyz.y, xyz.z, w);
  }

  /**
   * @param light - The light's place among the records.
   * @param face - Which of its shadow's faces, their vectors following `LIGHT_RECORD.faces`.
   * @param x - The vector's first float.
   * @param y - Its second.
   * @param z - Its third.
   * @param w - Its fourth.
   */
  public setFace(light: number, face: number, x: number, y: number, z: number, w: number): void {
    this.write((light * LIGHT_VECTORS + LIGHT_RECORD.faces + face) * 4, x, y, z, w);
  }

  /**
   * @param count - Lights the frame wrote, which alone are uploaded.
   */
  public upload(count: number): void {
    if (count > 0) {
      queueBufferUpload(this.buffer, 0, count * LIGHT_VECTORS * 4);
    }
  }

  private write(at: number, x: number, y: number, z: number, w: number): void {
    this.data[at] = x;
    this.data[at + 1] = y;
    this.data[at + 2] = z;
    this.data[at + 3] = w;
  }
}
