module.exports = {
  env: {
    node: true, // for "process"
    browser: true,
    es6: true,
  },
  parser: '@typescript-eslint/parser',
  plugins: [
    '@typescript-eslint',
    'react',
    'react-hooks',
    'prettier',
    'jsx-a11y',
  ],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:prettier/recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended',
    'plugin:jsx-a11y/recommended',
  ],
  globals: {
    Atomics: 'readonly',
    SharedArrayBuffer: 'readonly',
  },
  settings: {
    react: {
      pragma: 'React',
      version: 'detect',
    },
  },
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

    'no-console': 'warn',
    'no-unused-vars': 'off',
    'react/prop-types': 'off',
    'react-hooks/exhaustive-deps': [
      'error',
      {
        additionalHooks: 'useRecoilCallback',
      },
    ],
    'react/display-name': 'off',
    'jsx-a11y/no-autofocus': 'off',
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/no-explicit-any': ['error', { fixToUnknown: true }],
    '@typescript-eslint/explicit-function-return-type': [
      'error',
      { allowExpressions: true, allowTypedFunctionExpressions: true },
    ],
    '@typescript-eslint/no-object-literal-type-assertion': 'off',
    '@typescript-eslint/no-inferrable-types': [
      'error',
      { ignoreParameters: true },
    ],
    '@typescript-eslint/camelcase': 'off',
  },
  overrides: [
    {
      // Special config files
      files: ['*.js', '*.mjs'],
      parserOptions: {
        ecmaVersion: 3,
        project: null,
      },
      rules: {
        '@typescript-eslint/no-var-requires': 'off',
      },
    },
  ],
};
