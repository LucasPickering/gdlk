import React, { useEffect } from 'react';
import { Prompt } from 'react-router-dom';

/**
 * Show a prompt when the user navigates away from the current page. This covers
 * both in-app navigation (within react-router) and browser/out-of-app nav,
 * e.g. refreshing or manually changing the URL.
 *
 * @param when If passed, then the prompt will only be shown when this is true
 * @param message Message to show in the prompt, ONLY SHOWS FOR IN-APP NAV!
 *  Browsers don't allow customers messages for browser-level navigation.
 */
const PromptOnExit = ({
  when,
  message,
}: {
  when: boolean;
  message: string;
}): React.ReactElement => {
  // This handles browser nav (refresh, URL change, etc.)
  useEffect(() => {
    if (when) {
      const prompt = (event: BeforeUnloadEvent): void => {
        event.preventDefault();
        event.returnValue = ''; // Needed for chrome
      };

      window.addEventListener('beforeunload', prompt);

      return () => {
        window.removeEventListener('beforeunload', prompt);
      };
    }
  }, [when]);

  // This handles in-app navigation (just within react-router)
  return <Prompt when={when} message={message} />;
};

PromptOnExit.defaultProps = {
  when: true,
};

export default PromptOnExit;
