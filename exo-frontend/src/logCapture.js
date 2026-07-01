/**
 * logCapture.js
 * 
 * Intercepts console.log, console.error, console.warn, console.info, console.debug
 * and sends them to the server via WebSocket.
 */

export function setupLogCapture(socket) {
  const originalLog = console.log;
  const originalError = console.error;
  const originalWarn = console.warn;
  const originalInfo = console.info;
  const originalDebug = console.debug;

  function sendLogToServer(level, args) {
    try {
      if (socket && socket.readyState === WebSocket.OPEN) {
        const message = args
          .map((arg) => {
            if (typeof arg === 'object') {
              return JSON.stringify(arg);
            }
            return String(arg);
          })
          .join(' ');

        const logEntry = {
          type: 'log',
          level: level,
          message: message,
          timestamp: new Date().toISOString(),
        };
        socket.send(JSON.stringify(logEntry));
      }
    } catch (e) {
      // Silently fail if sending fails
    }
  }

  console.log = function (...args) {
    originalLog(...args);
    sendLogToServer('info', args);
  };

  console.error = function (...args) {
    originalError(...args);
    sendLogToServer('error', args);
  };

  console.warn = function (...args) {
    originalWarn(...args);
    sendLogToServer('warn', args);
  };

  console.info = function (...args) {
    originalInfo(...args);
    sendLogToServer('info', args);
  };

  console.debug = function (...args) {
    originalDebug(...args);
    sendLogToServer('debug', args);
  };
}
