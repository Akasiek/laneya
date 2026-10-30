(() => {
  const feedSocket = document.getElementById("feed-socket");
  const label = document.getElementById("feed-connection-label");
  const indicator = document.getElementById("feed-connection-indicator");
  // htmx-ext-ws already retries these close codes; avoid scheduling a second retry.
  const automaticRetryCodes = new Set([1006, 1011, 1012, 1013]);
  let retryTimer;

  htmx.config.wsReconnectDelay = (attempt) =>
    Math.min(1000 * 2 ** Math.min(attempt, 5), 30000);

  function setStatus(text, color) {
    label.textContent = text;
    indicator.style.color = color;
  }

  function reconnectAfterClose(socketWrapper) {
    clearTimeout(retryTimer);
    retryTimer = setTimeout(() => {
      if (feedSocket.isConnected) {
        socketWrapper.reconnect();
      }
    }, 1000);
  }

  feedSocket.addEventListener("htmx:wsConnecting", () => {
    clearTimeout(retryTimer);
    setStatus("Connecting…", "#f59e0b");
  });

  feedSocket.addEventListener("htmx:wsOpen", () => {
    clearTimeout(retryTimer);
    setStatus("Connected", "#22c55e");
  });

  feedSocket.addEventListener("htmx:wsClose", (event) => {
    const text = navigator.onLine
      ? "Disconnected — reconnecting…"
      : "Offline — waiting for network";
    setStatus(text, "#f59e0b");

    if (!automaticRetryCodes.has(event.detail.event.code)) {
      reconnectAfterClose(event.detail.socketWrapper);
    }
  });

  feedSocket.addEventListener("htmx:wsError", () => {
    setStatus("Connection error - reconnecting…", "#ef4444");
  });
})();
