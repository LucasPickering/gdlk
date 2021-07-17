import React from 'react';
import HardwareSpecDetails from './HardwareSpecDetails';
import { useParams } from 'react-router-dom';
import NotFoundPage from '@root/components/NotFoundPage';
import { hardware } from '@root/data/hardware';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();
  const hardwareSpec = hardware[hwSlug];

  if (hardwareSpec) {
    return <HardwareSpecDetails hardwareSpec={hardwareSpec} />;
  }

  return <NotFoundPage />;
};

export default HardwareSpecView;
