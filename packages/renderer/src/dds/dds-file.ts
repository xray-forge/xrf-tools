import { Nullable } from "@xrf/types";

import { DDS_BLOCK_SIZE } from "#/dds/dds-block-format";
import { getDdsDxgiLayout } from "#/dds/dds-dxgi";
import { getDdsFourCcLayout } from "#/dds/dds-fourcc";
import { DDS_DIMENSION_TEXTURE_3D, IDdsHeader, IDdsHeaderRead, readDdsHeader } from "#/dds/dds-header";
import { EDdsLayout, TDdsLayout } from "#/dds/dds-layout";
import { describeDdsMasks, getDdsMaskLayout } from "#/dds/dds-masks";
import { IDdsMipmap, IDdsMipmapChain, readDdsMipmaps } from "#/dds/dds-mipmaps";
import { EDdsRefusal, IDdsRefusal } from "#/dds/dds-refusal";

/** Faces a cubemap stores, `+x -x +y -y +z -z`, which is also the order its layers upload in. */
export const DDS_CUBE_FACES: number = 6;

/** A dds file, read. */
export interface IDdsFile {
  width: number;
  height: number;
  /**
   * Mips in order, largest first. A cubemap's hold every face of their level one after another, as its layers
   * upload, where the file stores each face's whole chain before the next face's.
   */
  mipmaps: Array<IDdsMipmap>;
  /** Whether it is six faces of a sky rather than one picture. */
  isCube: boolean;
  mipmapCount: number;
  layout: TDdsLayout;
}

/**
 * What a read came to: exactly one of the two is present.
 */
export interface IDdsRead {
  file: Nullable<IDdsFile>;
  refusal: Nullable<IDdsRefusal>;
}

/**
 * Reads a dds file, or says why it cannot be read.
 *
 * @param bytes - The file as read.
 * @returns The file, or the reason it was refused.
 */
export function readDdsFile(bytes: ArrayBuffer): IDdsRead {
  const read: IDdsHeaderRead = readDdsHeader(bytes);

  if (!read.header) {
    return { file: null, refusal: read.refusal };
  }

  const header: IDdsHeader = read.header;
  const layout: TDdsLayout | IDdsRefusal = toLayout(header);

  if ("reason" in layout) {
    return { file: null, refusal: layout };
  }

  // A cubemap missing a face is a broken file. One of texels the engine never ships: a sky is block compressed.
  if (header.cubemap && !header.cubemap.isWhole) {
    return refuse(EDdsRefusal.CUBEMAP, "the file is a cubemap missing faces");
  }

  if (header.cubemap && layout.kind !== EDdsLayout.BLOCK) {
    return refuse(EDdsRefusal.CUBEMAP, "the file is a cubemap of uncompressed texels");
  }

  const { width, height, mipmapCount } = header;

  // Refused rather than uploaded: WebGL's `compressedTexImage2D` answers `INVALID_OPERATION` and WebGPU invalidates the
  // texture for a base level narrower than its block, and a texture that failed to upload samples as black. Direct3D
  // takes these, which is why the game shows them and a browser does not. The backend expands them to png instead,
  // where the alpha survives.
  if (layout.kind === EDdsLayout.BLOCK && (width < DDS_BLOCK_SIZE || height < DDS_BLOCK_SIZE)) {
    return refuse(EDdsRefusal.SUB_BLOCK, `the picture is ${width}x${height}, under the ${DDS_BLOCK_SIZE} of a block`);
  }

  const chain: IDdsMipmapChain = { height, layout, mipmapCount, width };
  const isCube: boolean = header.cubemap !== null;
  const mipmaps: Nullable<Array<IDdsMipmap>> = isCube
    ? readCubeMipmaps(bytes, header.dataOffset, chain)
    : readDdsMipmaps(bytes, header.dataOffset, chain);

  if (!mipmaps) {
    return refuse(EDdsRefusal.TRUNCATED, "the file stops before the texels its header declares");
  }

  return { file: { height, isCube, layout, mipmapCount, mipmaps, width }, refusal: null };
}

/**
 * @param read - What a read came to.
 * @returns The file, for one a surface draws flat: read, and one picture rather than a sky's six faces.
 */
export function toDdsPicture(read: IDdsRead): Nullable<IDdsFile> {
  return read.file && !read.file.isCube ? read.file : null;
}

/** Every face's chain in turn, regrouped by level: each level's faces copied together, in face order. */
function readCubeMipmaps(bytes: ArrayBuffer, start: number, chain: IDdsMipmapChain): Nullable<Array<IDdsMipmap>> {
  const faces: Array<Array<IDdsMipmap>> = [];
  let offset: number = start;

  for (let face = 0; face < DDS_CUBE_FACES; face += 1) {
    const mipmaps: Nullable<Array<IDdsMipmap>> = readDdsMipmaps(bytes, offset, chain);

    if (!mipmaps) {
      return null;
    }

    faces.push(mipmaps);
    offset += mipmaps.reduce((total: number, mipmap: IDdsMipmap) => total + mipmap.data.byteLength, 0);
  }

  return faces[0].map((top: IDdsMipmap, level: number) => {
    const data: Uint8Array = new Uint8Array(top.data.byteLength * DDS_CUBE_FACES);

    faces.forEach((mipmaps: Array<IDdsMipmap>, face: number) =>
      data.set(mipmaps[level].data, face * top.data.byteLength)
    );

    return { data, height: top.height, width: top.width };
  });
}

/** The layout a header declares, from whichever of its three places declares it. */
function toLayout(header: IDdsHeader): TDdsLayout | IDdsRefusal {
  if (header.extended) {
    const { dimension, arraySize, dxgiFormat } = header.extended;

    // Only the two that would really misparse. A writer leaving either field zero means a plain 2d texture, and
    // refusing those would refuse files that read correctly.
    if (dimension === DDS_DIMENSION_TEXTURE_3D || arraySize > 1) {
      return {
        detail: `the file is a DX10 resource of dimension ${dimension} and array size ${arraySize}`,
        reason: EDdsRefusal.UNSUPPORTED_DIMENSION,
      };
    }

    return (
      getDdsDxgiLayout(dxgiFormat) ?? {
        detail: `DXGI_FORMAT ${dxgiFormat} is not modelled`,
        reason: EDdsRefusal.UNSUPPORTED_DXGI,
      }
    );
  }

  if (header.fourCc) {
    return (
      getDdsFourCcLayout(header.fourCc) ?? {
        detail: `the four character tag '${header.fourCc}' is not modelled`,
        reason: EDdsRefusal.UNSUPPORTED_FOURCC,
      }
    );
  }

  return (
    getDdsMaskLayout(header.masks) ?? { detail: describeDdsMasks(header.masks), reason: EDdsRefusal.UNSUPPORTED_MASKS }
  );
}

function refuse(reason: EDdsRefusal, detail: string): IDdsRead {
  return { file: null, refusal: { detail, reason } };
}
