(function (window) {
  function SimpleEmitter() {
    this.events = {};
  }
  SimpleEmitter.prototype.on = function (event, cb) {
    if (!this.events[event]) this.events[event] = [];
    this.events[event].push(cb);
  };
  SimpleEmitter.prototype.emit = function (event, data) {
    if (this.events[event]) {
      this.events[event].forEach(function (cb) {
        cb(data);
      });
    }
  };

  function RustWebSocketClient() {
    SimpleEmitter.call(this);
    var self = this;
    var protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    var wsUrl = protocol + "//" + window.location.host + "/ws";
    
    function connect() {
      var ws = new WebSocket(wsUrl);
      self.ws = ws;

      ws.onopen = function () {
        if (self._hasConnectedOnce) {
          self.emit("reconnect");
        }
        self._hasConnectedOnce = true;
      };

      ws.onmessage = function (event) {
        try {
          var payload = JSON.parse(event.data);
          if (payload.type) {
            if (payload.type === "say") {
              self.emit("say", payload.data);
            } else if (payload.type === "clear_history_client") {
              self.emit("clear_history_client");
            } else {
              // 对于 notice, system, userflush, image 等，原 node 版均调用 JSON.stringify 发送 string，chat.js 内部做 JSON.parse
              var dataArg = typeof payload.data === "string" ? payload.data : JSON.stringify(payload.data);
              self.emit(payload.type, dataArg);
            }
          }
        } catch (e) {
          console.error("WS Parse error:", e);
        }
      };


      ws.onclose = function () {
        self.emit("disconnect");
        setTimeout(connect, 3000);
      };

      ws.onerror = function (err) {
        console.error("WS Error:", err);
      };
    }

    this.sendEvent = function (eventType, data) {
      if (self.ws && self.ws.readyState === WebSocket.OPEN) {
        var envelope = {
          type: eventType,
          data: data
        };
        self.ws.send(JSON.stringify(envelope));
      }
    };

    connect();
  }

  RustWebSocketClient.prototype = Object.create(SimpleEmitter.prototype);
  RustWebSocketClient.prototype.constructor = RustWebSocketClient;

  RustWebSocketClient.prototype.emitServer = function (eventType, data) {
    if (eventType === "online" || eventType === "say" || eventType === "image" || eventType === "notice" || eventType === "clear_history" || eventType === "fileUpload" || eventType === "offline") {
      var parsedData = data;
      if (typeof data === "string") {
        try {
          parsedData = JSON.parse(data);
        } catch (e) {
          parsedData = data;
        }
      }
      this.sendEvent(eventType, parsedData);
    }
  };

  window.io = {
    connect: function () {
      var client = new RustWebSocketClient();
      client.emit = function (event, data) {
        if (event === "online" || event === "say" || event === "image" || event === "notice" || event === "clear_history" || event === "fileUpload" || event === "offline") {
          client.emitServer(event, data);
        } else {
          SimpleEmitter.prototype.emit.call(client, event, data);
        }
      };
      return client;
    }
  };
})(window);
