import { Typography } from "@material-ui/core";
import React, { useEffect, useState } from "react";

const Clock: React.FC = () => {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()));
    return () => clearInterval(id);
  }, [setNow]);

  return (
    <Typography>
      {now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
    </Typography>
  );
};

export default Clock;
