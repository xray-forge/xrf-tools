import { Nullable } from "@xrf/types";

import { DDS_BLOCK_SIZE } from "#/texture/dds/dds-block-format";
import { getDdsDxgiLayout } from "#/texture/dds/dds-dxgi";
import { getDdsFourCcLayout } from "#/texture/dds/dds-fourcc";
import { DDS_DIMENSION_TEXTURE_3D, IDdsHeader, IDdsHeaderRead, readDdsHeader } from "#/texture/dds/dds-header";
import { EDdsLayout, TDdsLayout } from "#/texture/dds/dds-layout";
import { describeDdsMasks, getDdsMaskLayout } from "#/texture/dds/dds-masks";
import { IDdsMipmap, readDdsMipmaps } from "#/texture/dds/dds-mipmaps";
import { EDdsRefusal, IDdsRefusal } from "#/texture/dds/dds-refusal";

/** A dds file, read. */
export interface IDdsFile {
  width: number;
  height: number;
  /** Mips in order, largest first. */
  mipmaps: Array<IDdsMipmap>;
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

  // Refused rather than read: a cubemap is six faces, and drawing one flat would stretch whichever face came first
  // over the surface. Whether its faces are all there is said in the detail, because a malformed cubemap is a broken
  // file while a whole one is simply the wrong kind of picture.
  if (header.cubemap) {
    return refuse(
      EDdsRefusal.CUBEMAP,
      `the file is a cubemap, ${header.cubemap.isWhole ? "six faces" : "missing faces"}`
    );
  }

  const { width, height, mipmapCount } = header;

  // Refused rather than uploaded: WebGL's `compressedTexImage2D` answers `INVALID_OPERATION` and WebGPU invalidates the
  // texture for a base level narrower than its block, and a texture that failed to upload samples as black. Direct3D
  // takes these, which is why the game shows them and a browser does not. The backend expands them to png instead,
  // where the alpha survives.
  if (layout.kind === EDdsLayout.BLOCK && (width < DDS_BLOCK_SIZE || height < DDS_BLOCK_SIZE)) {
    return refuse(EDdsRefusal.SUB_BLOCK, `the picture is ${width}x${height}, under the ${DDS_BLOCK_SIZE} of a block`);
  }

  const mipmaps: Nullable<Array<IDdsMipmap>> = readDdsMipmaps(bytes, header.dataOffset, {
    height,
    layout,
    mipmapCount,
    width,
  });

  if (!mipmaps) {
    return refuse(EDdsRefusal.TRUNCATED, "the file stops before the texels its header declares");
  }

  return { file: { height, layout, mipmapCount, mipmaps, width }, refusal: null };
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
