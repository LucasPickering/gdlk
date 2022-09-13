import React from "react";
import { IconButton } from "@material-ui/core";
import { Add as IconAdd, Remove as IconRemove } from "@material-ui/icons";
import { hardwareSpecState } from "@root/state/user";
import { useRecoilState } from "recoil";
import { HardwareSpec } from "@root/util/types";

const HardwareSpecField: React.FC<{
  field: keyof HardwareSpec;
  editing: boolean;
  min: number;
  max: number;
}> = ({ field, editing, min, max }) => {
  const [hardwareSpec, setHardwareSpec] = useRecoilState(hardwareSpecState);
  const value = hardwareSpec[field];

  if (!editing) {
    return <>{value}</>;
  }

  return (
    <>
      <IconButton
        disabled={value <= min}
        onClick={() =>
          setHardwareSpec((old) => ({
            ...old,
            [field]: old[field] - 1,
          }))
        }
      >
        <IconRemove />
      </IconButton>
      {value}
      <IconButton
        disabled={value >= max}
        onClick={() =>
          setHardwareSpec((old) => ({
            ...old,
            [field]: old[field] + 1,
          }))
        }
      >
        <IconAdd />
      </IconButton>
    </>
  );
};

export default HardwareSpecField;
