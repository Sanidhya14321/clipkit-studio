import { useMemo, useRef } from 'react';
import * as THREE from 'three';

const loaded = new Set<string>();
const loading = new Map<string, Promise<void>>();

export const useCanvasFont = (spec: string, ready: () => Promise<unknown>): void => {
  if (loaded.has(spec)) {
    return;
  }
  let pending = loading.get(spec);
  if (pending === undefined) {
    pending = ready()
      .then(() => document.fonts.load(spec))
      .then(() => {
        loaded.add(spec);
      });
    loading.set(spec, pending);
  }
  throw pending;
};

export const useCanvasTexture = (
  width: number,
  height: number,
  draw: (context: CanvasRenderingContext2D) => void,
  key: string,
): THREE.CanvasTexture => {
  const texture = useMemo(() => {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const map = new THREE.CanvasTexture(canvas);
    map.colorSpace = THREE.SRGBColorSpace;
    map.anisotropy = 16;
    return map;
  }, [width, height]);
  const drawn = useRef<{ texture: THREE.CanvasTexture; key: string } | null>(null);
  if (drawn.current?.texture !== texture || drawn.current.key !== key) {
    const context = (texture.image as HTMLCanvasElement).getContext('2d');
    if (context !== null) {
      context.clearRect(0, 0, width, height);
      draw(context);
      texture.needsUpdate = true;
    }
    drawn.current = { texture, key };
  }
  return texture;
};
