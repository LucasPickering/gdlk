import React from 'react';

export interface User {
  id: string;
  username: string;
}

export interface UserContextType {
  /**
   * Has the user logged in? This should NOT be used for most user checks.
   * The user could be logged in but not have initalized their user yet, which
   * will disallow most API requests that require a user.
   */
  loggedInUnsafe: boolean;

  /**
   * Is the user logged in and initialized? Convenience function to check if
   * the user field is populated.
   */
  loggedIn: boolean;

  /**
   * The logged-in and initialized user.
   */
  user: User | undefined;
}

export const UserContext = React.createContext<UserContextType>(
  {} as UserContextType // this default value never gets used so this is "safe"
);

export const defaultUserContext = {
  loggedInUnsafe: false,
  loggedIn: false,
  user: undefined,
};
