const path = require('path');
const ForkTsCheckerWebpackPlugin = require('fork-ts-checker-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');

module.exports = {
  mode: process.env.NODE_ENV || 'development',
  entry: './src/index.tsx',
  target: 'web',
  output: {
    path: path.resolve(__dirname, 'build'),
    publicPath: '/',
    filename: '[name].bundle.js',
  },

  experiments: {
    syncWebAssembly: true,
    topLevelAwait: true,
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        exclude: /node_modules/,
        loader: 'babel-loader',
      },
      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },
      {
        test: /\.js$/,
        enforce: 'pre',
        use: ['source-map-loader'],
      },
      {
        test: /\.wasm$/,
        include: path.resolve(__dirname, 'src'),
        use: [{ loader: require.resolve('wasm-loader'), options: {} }],
      },
    ],
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: 'public/index.html',
      favicon: 'public/favicon.ico',
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '../crates/wasm'),
      outDir: path.resolve(__dirname, '../crates/wasm/pkg'),
      outName: 'gdlk_wasm',
      watchDirectories: [
        path.resolve(__dirname, '../crates/core/src'),
        path.resolve(__dirname, '../crates/wasm/src'),
      ],
    }),
    // Run typechecking in a separate process (cause babel doesn't do it)
    new ForkTsCheckerWebpackPlugin(),
    new BundleAnalyzerPlugin({
      analyzerMode: process.env.WEBPACK_BUNDLE_ANALYZER_MODE || 'disabled',
    }),
  ],

  resolve: {
    modules: [path.resolve(__dirname, 'src'), 'node_modules'],
    extensions: ['.tsx', '.ts', '.js'],
  },

  optimization: {
    splitChunks: {
      cacheGroups: {
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          priority: -20,
          name: 'vendors',
          chunks: 'all',
          filename: '[name].app.bundle.js',
        },
      },
    },
  },

  watchOptions: {
    ignored: /node_modules/,
  },
  devServer: {
    host: process.env.WEBPACK_HOST,
    port: 3000,
    // https: true, // Needed for oauth
    contentBase: path.join(__dirname, 'public'),
    historyApiFallback: true,
    watchContentBase: true,
    hot: true,
    proxy: {
      '/api': process.env.GDLK_API_HOST,
    },
  },
};
