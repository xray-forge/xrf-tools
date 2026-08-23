import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { IMediaVolume, useMediaVolume } from "@/core/ui/media/use-media-volume";

describe("useMediaVolume", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("starts at full volume when nothing was ever chosen", () => {
    expect(renderHook(() => useMediaVolume()).result.current.value).toBe(1);
  });

  it("persists a chosen level so the next player starts there", () => {
    const { result } = renderHook(() => useMediaVolume());

    act(() => result.current.set(0.5));

    expect(result.current.value).toBe(0.5);
    // A separate instance stands in for the player remounting on the next selection, which is what used to lose it.
    expect(renderHook(() => useMediaVolume()).result.current.value).toBe(0.5);
  });

  it("keeps a silenced player silent rather than treating zero as unset", () => {
    const { result } = renderHook(() => useMediaVolume());

    act(() => result.current.set(0));

    expect(renderHook(() => useMediaVolume()).result.current.value).toBe(0);
  });

  it("holds levels inside the range a media element accepts", () => {
    const { result } = renderHook(() => useMediaVolume());

    act(() => result.current.set(4));
    expect(result.current.value).toBe(1);

    act(() => result.current.set(-1));
    expect(result.current.value).toBe(0);
  });

  it("falls back to full volume rather than silence when the stored value is unusable", () => {
    // Silence recovered from a corrupt value looks like broken playback, and sends the user hunting for a mute button.
    for (const stored of ["", "loud", "2", "-0.5", "NaN"]) {
      window.localStorage.setItem("xrf.media.volume", stored);

      const volume: IMediaVolume = renderHook(() => useMediaVolume()).result.current;

      expect(volume.value).toBe(1);
    }
  });
});
