import type { FC, ReactElement } from 'react';
import type { ClipId } from '@/schemas';

export type SceneModule = {
  id: ClipId;
  component: FC<never>;
  render: (key: string) => ReactElement;
};
