import { useSyncExternalStore } from "react";
export function createPlaybackClock() {
  let seconds = 0;
  const listeners = new Set<() => void>();
  return {
    get: () => seconds,
    subscribe: (listener: () => void) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    set: (next: number) => {
      if (seconds === next) return;
      seconds = next;
      listeners.forEach((listener) => listener());
    },
  };
}
export type PlaybackClock = ReturnType<typeof createPlaybackClock>;
export function ClockTime({
  clock,
  instrument = false,
}: {
  clock: PlaybackClock;
  instrument?: boolean;
}) {
  const seconds = useSyncExternalStore(clock.subscribe, clock.get);
  return (
    <span className="timecode" data-ck-playhead={instrument ? "" : undefined}>
      {seconds.toFixed(3)} s
    </span>
  );
}
