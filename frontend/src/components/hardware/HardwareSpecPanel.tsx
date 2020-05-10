import React from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { HardwareSpecPanel_hardwareSpec } from './__generated__/HardwareSpecPanel_hardwareSpec.graphql';
import {
  ExpansionPanel,
  Typography,
  ExpansionPanelSummary,
  ExpansionPanelDetails,
  List,
  ListItem,
  ListItemText,
  makeStyles,
} from '@material-ui/core';
import { ExpandMore as IconExpandMore } from '@material-ui/icons';
// import Link from 'components/common/Link';
import { Link } from 'react-router-dom';
import HardwareSpecSummary from './HardwareSpecSummary';

const useLocalStyles = makeStyles(({ spacing }) => ({
  panelHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  panelBody: {
    display: 'flex',
    justifyContent: 'space-between',
  },

  hwStats: {
    borderCollapse: 'collapse',
  },
  hwStatName: {
    textAlign: 'left',
    paddingRight: spacing(2),
  },
  hwStatValue: {
    textAlign: 'right',
  },
}));

const HardwareSpecPanel: React.FC<{
  hardwareSpec: HardwareSpecPanel_hardwareSpec;
  relay: RelayProp;
}> = ({ hardwareSpec }) => {
  const localClasses = useLocalStyles();

  return (
    <ExpansionPanel>
      <ExpansionPanelSummary
        classes={{ content: localClasses.panelHeader }}
        expandIcon={<IconExpandMore />}
        aria-controls={`hardware-${hardwareSpec.slug}-content`}
        id={`hardware-${hardwareSpec.slug}-header`}
      >
        <Typography variant="h5">{hardwareSpec.slug}</Typography>

        <Typography variant="body1">
          {hardwareSpec.programSpecs.totalCount} puzzles
        </Typography>
      </ExpansionPanelSummary>

      <ExpansionPanelDetails classes={{ root: localClasses.panelBody }}>
        <HardwareSpecSummary hardwareSpec={hardwareSpec} />

        <List dense>
          {hardwareSpec.programSpecs.edges.map(({ node: programSpec }) => {
            return (
              <ListItem
                key={programSpec.slug}
                button
                component={Link}
                // This prop gets forwarded to the link
                to={`/hardware/${hardwareSpec.slug}/puzzles/${programSpec.slug}`}
              >
                <ListItemText
                  primary={programSpec.slug}
                  secondary={`${programSpec.userPrograms.totalCount} solutions`}
                />
              </ListItem>
            );
          })}
        </List>
      </ExpansionPanelDetails>
    </ExpansionPanel>
  );
};

export default createFragmentContainer(HardwareSpecPanel, {
  hardwareSpec: graphql`
    fragment HardwareSpecPanel_hardwareSpec on HardwareSpecNode {
      slug
      ...HardwareSpecSummary_hardwareSpec
      programSpecs(first: 5) {
        totalCount
        edges {
          node {
            slug
            userPrograms {
              totalCount
            }
          }
        }
      }
    }
  `,
});
