import { puzzles } from '@root/data/puzzles';
import React from 'react';
import NavMenu from '../common/NavMenu';
import PuzzleList from '../puzzle/PuzzleList';

const MainMenu: React.FC = () => (
  <NavMenu
    items={[
      {
        label: 'Puzzles',
        children: <PuzzleList puzzles={Object.values(puzzles)} />,
      },
      { label: 'GDLK Language Reference', to: '/docs' },
    ]}
  />
);

export default MainMenu;
