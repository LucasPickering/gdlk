const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const WASM_EXT_RGX = /\.wasm$/;

module.exports = function override(config) {
  config.resolve.extensions.push('.wasm');

  // Make file-loader ignore file types that we customize
  const oneOfRule = config.module.rules.find((rule) => Boolean(rule.oneOf));
  if (oneOfRule) {
    const fileLoaderRule = oneOfRule.oneOf.find((rule) =>
      Boolean(rule.loader && rule.loader.includes('file-loader'))
    );
    fileLoaderRule.exclude.push(WASM_EXT_RGX);
  }

  // Add markdown and wasm loaders
  config.module.rules.push({
    test: WASM_EXT_RGX,
    include: path.resolve(__dirname, 'src'),
    use: [{ loader: require.resolve('wasm-loader'), options: {} }],
  });

  config.plugins.push(
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '../crates/wasm'),
      outDir: path.resolve(__dirname, '../crates/wasm/pkg'),
      outName: 'gdlk_wasm',
      watchDirectories: [
        path.resolve(__dirname, '../crates/core/src'),
        path.resolve(__dirname, '../crates/wasm/src'),
      ],
    })
  );

  return config;
};
