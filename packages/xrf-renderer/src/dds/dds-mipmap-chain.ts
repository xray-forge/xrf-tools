import { TDdsLayout } from "#/dds/dds-layout";

/** What a mip walk needs to know about the file it is walking. */
export interface IDdsMipmapChain {
  width: number;
  height: number;
  mipmapCount: number;
  layout: TDdsLayout;
}
