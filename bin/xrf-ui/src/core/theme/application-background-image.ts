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
  const ellipse: string = `ellipse ${APPLICATION_BACKGROUND.radiusX}px ${APPLICATION_BACKGROUND.radiusY}px`;

  return [
    `radial-gradient(${ellipse} at ${APPLICATION_BACKGROUND.secondary.x}px ${APPLICATION_BACKGROUND.secondary.y}px, ` +
      `color-mix(in srgb, ${secondary} ${secondaryOpacity}, transparent), transparent)`,
    `radial-gradient(${ellipse} at ${APPLICATION_BACKGROUND.primary.x}px ${APPLICATION_BACKGROUND.primary.y}px, ` +
      `color-mix(in srgb, ${primary} ${primaryOpacity}, transparent), transparent)`,
  ].join(", ");
}
