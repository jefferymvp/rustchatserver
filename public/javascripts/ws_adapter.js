(function (window) {
  function SimpleEmitter() {
    this.events = {};
  }
  SimpleEmitter.prototype.on = function (event, cb) {
    if (!this.events[event]) this.events[event] = [];
    this.events[event].push(cb);
  };
  SimpleEmitter.prototype.emitLocal = function (event, data) {
    if (this.events[event]) {
      this.events[event].forEach(function (cb) {
        try {
          cb(data);
        } catch (err) {
          console.error("[WS Adapter] 回调执行异常:", event, err);
        }
      });
    }
  };

  function RustWebSocketClient() {
    SimpleEmitter.call(this);
    var self = this;
    var protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    var wsUrl = protocol + "//" + window.location.host + "/ws";
    self.sendQueue = [];
    self._hasConnectedOnce = false;

    function connect() {
      console.log("[WS] 正在连接:", wsUrl);
      var ws = new WebSocket(wsUrl);
      self.ws = ws;
      window._dbgWs = ws;

      ws.onopen = function () {
        console.log("[WS] 连接成功! readyState:", ws.readyState, "待发队列长度:", self.sendQueue.length);
        // 连接建立后冲刷待发队列中的所有消息
        while (self.sendQueue.length > 0) {
          var item = self.sendQueue.shift();
          try {
            console.log("[WS] 发送队列消息:", item.type);
            self.ws.send(JSON.stringify(item));
          } catch (err) {
            console.error("[WS] 冲刷待发队列失败:", err);
          }
        }

        if (self._hasConnectedOnce) {
          self.emitLocal("reconnect");
        }
        self._hasConnectedOnce = true;
      };

      ws.onmessage = function (event) {
        try {
          var payload = JSON.parse(event.data);
          if (payload && payload.type) {
            console.log("[WS 收到服务端广播]", payload.type, payload.data);
            if (payload.type === "say") {
              // say 消息直接传递 SayMsg 对象 { time, data: { from, to, msg }, notice }
              self.emitLocal("say", payload.data);
            } else if (payload.type === "clear_history_client") {
              self.emitLocal("clear_history_client");
            } else {
              // 对于 notice, system, userflush, image 等，chat.js 内部使用 JSON.parse 解析
              var dataArg = typeof payload.data === "string" ? payload.data : JSON.stringify(payload.data);
              self.emitLocal(payload.type, dataArg);
            }
          }
        } catch (e) {
          console.error("[WS] 消息解析失败:", e, event.data);
        }
      };

      ws.onclose = function () {
        console.warn("[WS] 连接断开，准备重连...");
        self.emitLocal("disconnect");
        setTimeout(connect, 3000);
      };

      ws.onerror = function (err) {
        console.error("[WS 错误]:", err);
      };
    }

    this.sendServer = function (eventType, data) {
      var parsedData = data;
      if (typeof data === "string") {
        try {
          parsedData = JSON.parse(data);
        } catch (e) {
          parsedData = data;
        }
      }

      var envelope = {
        type: eventType,
        data: parsedData
      };

      if (self.ws && self.ws.readyState === WebSocket.OPEN) {
        console.log("[WS 发送至服务端]", eventType, parsedData);
        self.ws.send(JSON.stringify(envelope));
      } else {
        console.warn("[WS 暂存待发]", eventType);
        self.sendQueue.push(envelope);
      }
    };

    connect();
  }

  RustWebSocketClient.prototype = Object.create(SimpleEmitter.prototype);
  RustWebSocketClient.prototype.constructor = RustWebSocketClient;

  // 客户端业务层调用的 emit（向服务端发送）
  RustWebSocketClient.prototype.emit = function (eventType, data) {
    this.sendServer(eventType, data);
  };

  window.io = {
    connect: function () {
      return new RustWebSocketClient();
    }
  };
})(window);
