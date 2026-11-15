import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withViewportPane } from "#/testing/stories";

import { ImageViewport } from "./ImageViewport";

const TEXTURE_SIZE: number = 256;

/** Four concrete slabs, one of them mostly alpha, so the checkerboard shows where a texture is transparent. */
const CONCRETE_TEXTURE: string = `data:image/svg+xml,${encodeURIComponent(
  `<svg xmlns="http://www.w3.org/2000/svg" width="${TEXTURE_SIZE}" height="${TEXTURE_SIZE}">
    <path d="M0 0H256V256H128V128H0Z" fill="#7d7a73"/>
    <rect y="128" width="128" height="128" fill="#7d7a73" fill-opacity="0.35"/>
    <path d="M0 128H256M128 0V256" stroke="#5c5a55" stroke-width="4"/>
    <path d="M20 40l30 12 18-6M150 180l40 20 30-8M170 30l12 26" stroke="#4b4944" stroke-width="2" fill="none"/>
    <circle cx="190" cy="80" r="44" fill="#3f4a2e" fill-opacity="0.55"/>
  </svg>`
)}`;

const meta = {
  title: "Components/Media/ImageViewport",
  component: ImageViewport,
  decorators: [withViewportPane],
  args: {
    src: CONCRETE_TEXTURE,
    alt: "textures\\ston\\ston_beton05.dds",
    width: TEXTURE_SIZE,
    height: TEXTURE_SIZE,
    onContentClick: fn(),
  },
} satisfies Meta<typeof ImageViewport>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Opens fitted; the wheel zooms about the pointer, a drag pans, and the bar steps about the middle. */
export const Texture: TStory = {};

/** The second of a pair, which the first one's bar moves. */
export const WithoutControls: TStory = {
  args: { hasControls: false },
};
