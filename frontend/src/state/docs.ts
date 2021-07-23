import React from 'react';

/**
 * This context tells different sections of the docs which capabilities are
 * available, so that they can selectively show and hide content. For example,
 * we want to hide all references to stacks for machines that do not have any
 * stacks installed.
 */
export interface DocsContextType {
  showStacks: boolean;
}

export const DocsContext = React.createContext<DocsContextType>({
  showStacks: false,
});
