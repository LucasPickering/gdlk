import queryString, { ParsedQuery } from "query-string";
import { useLocation } from "react-router-dom";

/**
 * Parse the query params in the current route and return them.
 * TODO delete this (and remove query-string)
 */
const useRouteQuery = (): ParsedQuery<string | boolean | number> => {
  const { search } = useLocation();
  return queryString.parse(search);
};

export default useRouteQuery;
