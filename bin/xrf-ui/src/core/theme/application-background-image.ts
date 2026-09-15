import { APPLICATION_BACKGROUND } from "./tokens";

interface IApplicationBackgroundColors {
  primary: string;
  secondary: string;
  primaryOpacity: string;
  secondaryOpacity: string;
}

/** Shared gradient formula for the preload canvas and themed React surfaces. */
export function getApplicationBackgroundImage({
  primary,
  secondary,
  primaryOpacity,
  secondaryOpacity,
}: IApplicationBackgroundColors): string {
  return (
    `linear-gradient(${APPLICATION_BACKGROUND.angle}, ` +
    `color-mix(in srgb, ${secondary} ${secondaryOpacity}, transparent), ` +
    `color-mix(in srgb, ${primary} ${primaryOpacity}, transparent))`
  );
}
