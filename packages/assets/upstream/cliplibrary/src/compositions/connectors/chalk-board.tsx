import type { ReactNode } from 'react';
import { AbsoluteFill } from 'remotion';
import type { ConnectorChalk, PromoLook, PromoMotion } from '@/schemas';
import { Backdrop } from '../promo/backdrop';
import { TransitionFrame } from '../promo/transition-frame';
import { ChalkFilter } from './brush';

const ChalkBoard = ({ look }: { look: PromoLook }) => {
  const spacing = look.paperSpacing;
  const crossX = spacing / 2 - 0.75;
  const crossY = spacing * 0.6 + spacing / 2 - 0.75;
  return (
    <AbsoluteFill>
      <Backdrop look={look} />
      {look.paper === 'grid' ? (
        <AbsoluteFill
          style={{
            backgroundImage: `radial-gradient(circle, color-mix(in oklab, ${look.ink} 26%, transparent) 1.4px, transparent 2.6px)`,
            backgroundSize: `${spacing}px ${spacing}px`,
            backgroundPosition: `${crossX}px ${crossY}px`,
          }}
        />
      ) : null}
      <svg
        width="100%"
        height="100%"
        style={{ position: 'absolute', inset: 0, opacity: 0.09, mixBlendMode: 'screen' }}
        aria-hidden="true"
      >
        <filter id="chalk-dust">
          <feTurbulence type="fractalNoise" baseFrequency="0.004 0.012" numOctaves={3} seed={21} />
          <feColorMatrix type="saturate" values="0" />
        </filter>
        <rect width="100%" height="100%" filter="url(#chalk-dust)" />
      </svg>
    </AbsoluteFill>
  );
};

export const ChalkLayer = ({
  look,
  motion,
  chalk,
  filterId,
  children,
}: {
  look: PromoLook;
  motion: PromoMotion;
  chalk: ConnectorChalk;
  filterId: string;
  children: ReactNode;
}) => (
  <TransitionFrame motion={motion}>
    {look.showBackdrop ? <ChalkBoard look={look} /> : null}
    <ChalkFilter id={filterId} roughness={chalk.roughness} dryness={chalk.dryness} />
    {children}
  </TransitionFrame>
);
