/** `D3D10_RESOURCE_DIMENSION_TEXTURE3D`, a volume, which no surface draws. */
export const DDS_DIMENSION_TEXTURE_3D: number = 4;

/** What the `DX10` extended header says about the resource. */
export interface IDdsExtendedHeader {
  dxgiFormat: number;
  dimension: number;
  arraySize: number;
}
