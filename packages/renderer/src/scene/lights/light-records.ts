import { StorageBufferAttribute, Vector3 } from "three/webgpu";

import { LIGHT_RECORD, LIGHT_VECTORS, MAX_LIGHTS } from "#/scene/lights/light-record";

/** A vector of a light's record, as `LIGHT_RECORD` places it. */
export type TLightRecordVector = (typeof LIGHT_RECORD)[keyof typeof LIGHT_RECORD];

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
  public set(light: number, vector: number, x: number, y: number, z: number, w: number): void {
    const at: number = (light * LIGHT_VECTORS + vector) * 4;

    this.data[at] = x;
    this.data[at + 1] = y;
    this.data[at + 2] = z;
    this.data[at + 3] = w;
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
   * @param light - A light's place among the records.
   * @returns Its floats, as a view of the records.
   */
  public read(light: number): Float32Array {
    return this.data.subarray(light * LIGHT_VECTORS * 4, (light + 1) * LIGHT_VECTORS * 4);
  }

  /**
   * @param count - Lights the frame wrote, which alone are uploaded.
   */
  public upload(count: number): void {
    if (count > 0) {
      this.buffer.clearUpdateRanges();
      this.buffer.addUpdateRange(0, count * LIGHT_VECTORS * 4);
      this.buffer.needsUpdate = true;
    }
  }
}
