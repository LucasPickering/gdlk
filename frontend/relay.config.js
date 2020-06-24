module.exports = {
  src: './src',
  schema: '../api/schema.graphql',
  language: 'typescript',
  exclude: ['node_modules/**', '**/__mocks__/**', '**/__generated__/**'],
  customScalars: {
    // We'll need to manually convert to Date until https://github.com/facebook/relay/issues/91
    DateTimeUtc: 'string',
  },
};
