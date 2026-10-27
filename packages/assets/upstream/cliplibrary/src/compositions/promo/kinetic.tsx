import type { CSSProperties, ReactNode } from 'react';
import type { PromoLook, PromoMotion, PromoPlacement } from '@/schemas';
import { kineticFont } from './fonts';
import { PromoLayer, placementFrame } from './promo-layer';

export const displayType = (size: number, weight: number, tracking = -0.02): CSSProperties => ({
  fontFamily: kineticFont,
  fontSize: size,
  fontWeight: weight,
  letterSpacing: `${tracking}em`,
  lineHeight: 1,
  whiteSpace: 'pre',
});

const ALIGN_ITEMS: Record<PromoPlacement['horizontal'], CSSProperties['alignItems']> = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
};

export const KineticLayer = ({
  look,
  motion,
  placement,
  stage,
  children,
}: {
  look: PromoLook;
  motion: PromoMotion;
  placement: PromoPlacement;
  stage?: ReactNode;
  children: ReactNode;
}) => (
  <PromoLayer look={look} motion={motion}>
    {stage}
    <div style={placementFrame(placement)}>
      <div
        style={{
          width: placement.width,
          display: 'flex',
          flexDirection: 'column',
          alignItems: ALIGN_ITEMS[placement.horizontal],
          textAlign: placement.horizontal,
          position: 'relative',
        }}
      >
        {children}
      </div>
    </div>
  </PromoLayer>
);
