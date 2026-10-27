import type { CSSProperties, ReactNode } from 'react';
import type { PromoLook, PromoMotion, PromoPlacement } from '@/schemas';
import { Backdrop } from './backdrop';

export { isDarkLook } from './backdrop';

import { TransitionFrame } from './transition-frame';

export const PromoLayer = ({
  look,
  motion,
  focusX = 50,
  children,
}: {
  look: PromoLook;
  motion: PromoMotion;
  focusX?: number;
  children: ReactNode;
}) => (
  <TransitionFrame motion={motion}>
    {look.showBackdrop ? <Backdrop look={look} focusX={focusX} /> : null}
    {children}
  </TransitionFrame>
);

const JUSTIFY: Record<PromoPlacement['vertical'], CSSProperties['justifyContent']> = {
  top: 'flex-start',
  center: 'center',
  bottom: 'flex-end',
};

const ALIGN: Record<PromoPlacement['horizontal'], CSSProperties['alignItems']> = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
};

export const placementFrame = (placement: PromoPlacement): CSSProperties => ({
  position: 'absolute',
  inset: 0,
  display: 'flex',
  flexDirection: 'column',
  justifyContent: JUSTIFY[placement.vertical],
  alignItems: ALIGN[placement.horizontal],
  padding: `${placement.insetY}px ${placement.insetX}px`,
});
