import React from "react";
import { makeStyles, Typography } from "@material-ui/core";
import clsx from "clsx";

const useLocalStyles = makeStyles(({ spacing }) => ({
  simpleTable: {
    borderCollapse: "collapse",
  },
  simpleTableLabel: {
    textAlign: "left",
    paddingRight: spacing(4),
  },
  simpleTableValue: {
    textAlign: "right",
  },
}));

interface TableRow {
  label: string | number;
  value: React.ReactChild;
  action?: React.ReactChild;
}

/**
 * A horizontal table with a simple set of labels+values. The table will have
 * two columns. The first shows labels, the second shows the corresponding values.
 * Each row can optionally have an action button as well.
 * @param data The data rows
 */
const SimpleTable: React.FC<{
  className?: string;
  data: TableRow[];
}> = ({ className, data }) => {
  const localClasses = useLocalStyles();

  return (
    <table className={clsx(localClasses.simpleTable, className)}>
      <tbody>
        {data.map(({ label, value, action }) => {
          return (
            <tr key={label}>
              <th className={localClasses.simpleTableLabel}>
                <Typography component="span">{label}</Typography>
              </th>
              <td className={localClasses.simpleTableValue}>
                <Typography component="span">{value}</Typography>
              </td>
              {action && (
                <td className={localClasses.simpleTableValue}>{action}</td>
              )}
            </tr>
          );
        })}
      </tbody>
    </table>
  );
};

export default SimpleTable;
