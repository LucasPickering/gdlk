import { currencyState } from "@root/state/user";
import { formatCurrency } from "@root/util/format";
import React from "react";
import { useRecoilValue } from "recoil";

/**
 * Display the user's money
 */
const Wallet: React.FC = () => {
  const currency = useRecoilValue(currencyState);
  return <span>{formatCurrency(currency)}</span>;
};

export default Wallet;
