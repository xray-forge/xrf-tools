import { ArchiveSoundEnvironment } from "@/core/ipc/types/xrf-app";
import { formatSeconds } from "@/lib/format/duration";
import { formatNumber } from "@/lib/format/number";

/**
 * How a space sounds, taken over the two figures a listener hears first.
 *
 * @param environment - Preset to describe.
 * @returns Its decay and its apparent size.
 */
export function describeSpace(environment: ArchiveSoundEnvironment): string {
  return `${formatSeconds(environment.decayTime)} decay · ${formatNumber(environment.environmentSize, 1)} m`;
}

/**
 * What the preset adds at low and at high frequencies.
 *
 * @param environment - Preset to describe.
 * @returns The two room levels, with the EAX preset it stands for where it names one.
 */
export function describeLevels(environment: ArchiveSoundEnvironment): string {
  const room: string = formatNumber(environment.room, 0);
  const roomHf: string = formatNumber(environment.roomHf, 0);
  const levels: string = `room ${room} · high ${roomHf}`;

  return environment.environment === null ? levels : `${levels} · EAX preset ${environment.environment}`;
}
