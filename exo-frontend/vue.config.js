const { defineConfig } = require('@vue/cli-service')

module.exports = defineConfig({
  transpileDependencies: true,
  devServer: {
    proxy: {
      '/ws': {
        target: 'ws://localhost:8000', // Replace with your backend WebSocket server's URL
        ws: true, // This tells the proxy to handle WebSocket connections
        changeOrigin: true,
      },
    },
  },
})