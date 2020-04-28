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
import Link from 'components/common/Link';

const useLocalStyles = makeStyles(() => ({
  panelHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  panelBody: {
    display: 'flex',
    justifyContent: 'space-between',
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
          {hardwareSpec.programSpecs.totalCount} programs
        </Typography>
      </ExpansionPanelSummary>

      <ExpansionPanelDetails classes={{ root: localClasses.panelBody }}>
        <table>
          <tbody>
            <tr>
              <th>Registers</th>
              <td>{hardwareSpec.numRegisters}</td>
            </tr>
            <tr>
              <th>Stacks</th>
              <td>{hardwareSpec.numStacks}</td>
            </tr>
            <tr>
              <th>Stack Size</th>
              <td>{hardwareSpec.maxStackLength}</td>
            </tr>
          </tbody>
        </table>

        <List dense>
          {hardwareSpec.programSpecs.edges.map(({ node: programSpec }) => {
            return (
              <ListItem
                key={programSpec.slug}
                button
                component={Link}
                // These props get forwarded to the link
                styled={false}
                to={`/hardware/${hardwareSpec.slug}/programs/${programSpec.slug}`}
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
      numRegisters
      numStacks
      maxStackLength
      programSpecs(first: 5) {
        totalCount
        edges {
          node {
            slug
            input
            expectedOutput
            userPrograms {
              totalCount
            }
          }
        }
      }
    }
  `,
});
