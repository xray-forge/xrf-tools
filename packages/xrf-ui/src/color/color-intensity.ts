/** A colour's red, green and blue, unbounded above as an engine colour is. */
export type TColorChannels = readonly [number, number, number];

/** A colour as a picker holds it: each channel from 0 to 255. */
export interface IPickedColor {
  r: number;
  g: number;
  b: number;
}

/**
 * @param channels - A colour, as bright as the engine lets it be.
 * @returns What its brightest channel goes past one by, or one for a colour within the picker's range.
 */
export function toColorIntensity(channels: TColorChannels): number {
  return Math.max(1, ...channels);
}

/**
 * @param channels - A colour, as bright as the engine lets it be.
 * @param intensity - What it is scaled down by to fit the picker.
 * @returns It as the picker shows it.
 */
export function toPickedColor(channels: TColorChannels, intensity: number): IPickedColor {
  const [r, g, b] = channels.map((channel: number) => Math.round((Math.max(0, channel) / intensity) * 255));

  return { b, g, r };
}

/**
 * @param picked - A colour the picker chose.
 * @param intensity - What it is scaled up by.
 * @returns The colour as the engine takes it.
 */
export function fromPickedColor(picked: IPickedColor, intensity: number): TColorChannels {
  return [(picked.r / 255) * intensity, (picked.g / 255) * intensity, (picked.b / 255) * intensity];
}

/**
 * @param channels - A colour.
 * @param intensity - Its brightness now.
 * @param next - The brightness asked for.
 * @returns The same colour at that brightness.
 */
export function rescaleColor(channels: TColorChannels, intensity: number, next: number): TColorChannels {
  const scale: number = intensity > 0 ? next / intensity : 0;

  return [channels[0] * scale, channels[1] * scale, channels[2] * scale];
}
