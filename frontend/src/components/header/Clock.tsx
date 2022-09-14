import React, { useEffect, useState } from "react";

/**
 * Display the current time
 */
const Clock: React.FC = () => {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, [setNow]);

  return (
    <span>
      {now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
    </span>
  );
};

export default Clock;
