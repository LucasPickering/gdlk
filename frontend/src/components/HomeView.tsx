import React from 'react';
import ButtonLink from './common/ButtonLink';
import PageContainer from './common/PageContainer';

const HomeView: React.FC = () => {
  return (
    <PageContainer>
      <ButtonLink to="/hardware/hw1" variant="contained" color="primary">
        Go to HArdware
      </ButtonLink>
    </PageContainer>
  );
};

export default HomeView;
