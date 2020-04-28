const { createProxyMiddleware } = require('http-proxy-middleware');

const target = process.env.GDLK_API_HOST;
if (!target) {
  throw new Error('No proxy target defined. Set GDLK_API_HOST.');
}

module.exports = function (app) {
  app.use(createProxyMiddleware('/api', { target }));
  app.use(createProxyMiddleware('/ws', { target, ws: true }));
};
