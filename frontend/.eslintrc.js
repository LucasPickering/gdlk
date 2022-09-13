module.exports = {
  extends: ['@lucaspickering/eslint-config/react'],
  rules: {
    'no-restricted-syntax': [
      'error',
      {
        // Plain import statements on wasm imports "work", but they create weird
        // issues with webpack and force us to use React Suspense, so best to
        // just avoid them
        selector:
          'ImportDeclaration[importKind="value"][source.value="gdlk_wasm"]',
        message:
          'Use `import type` or `const ... = await import(...)` for Wasm imports',
      },
      {
        // The React Router and Material UI links don't have the right styling,
        // make sure we always use the local one
        selector:
          'ImportDeclaration[source.value=/@material-ui.core|react-router-dom/] > ImportSpecifier[imported.name="Link"]',
        message: 'Use the local `Link` component',
      },
    ],
  },
};
