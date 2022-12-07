const { defineConfig } = require('@vue/cli-service')
// const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;
module.exports = defineConfig({
  transpileDependencies: true,
  publicPath: '',
  configureWebpack: {
  //   plugins: [new BundleAnalyzerPlugin()],
    optimization: {
      splitChunks: {
        minSize: 10000,
        maxSize: 250000,
        cacheGroups: {
          vendors: {
            test: /[\\/]node_modules[\\/]/,
            name: 'vendor',
            chunks: 'all',
          },
        }
      }
    }
  }
})
