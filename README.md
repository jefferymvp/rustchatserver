# Chatroom (Rust 版高并发实时聊天室)

本项目为 `chatroom` 从最初的 Node.js + Express + Socket.io 架构完全重构为基于 **Rust (Actix-web + Actix-ws / WebSocket)** 的现代化高性能实时聊天系统。

---

## 🌟 特性概览

- **全异步高并发**：基于 Rust + Tokio + Actix-ws，内存开销极低、全双工低延迟。
- **超大文件传输支持**：默认支持高达 **100MB** 的文件、压缩包及高清图片上传。
- **业务特性 100% 完整保留**：
  - 用户登录与 Cookie 会话拦截 (`GET /`, `GET /signin`, `POST /signin`)；
  - 在线用户列表实时刷新 (`userflush`) 与上线/下线系统通知 (`system`)；
  - 公聊 (`to: all`) 与双击私聊 (`to: user`)；
  - 聊天输入框 `@` 用户自动联想；
  - 消息历史回放（保留最近 30 条公聊记录）；
  - **Botty 监控播报智能回放**：Status Report 仅保留最新 1 条、Found 掉落记录全部保留并按需回放；
  - 剪贴板图片直接粘贴上传 (`image`) 与文件二进制流上传并自动持久化保存到 `/doc`；
  - 全局历史一键清空指令 (`clear_history`) 广播；
  - 上线与新消息提示音效 (`play_ring`)。
- **现代化 WebSocket 适配**：采用轻量级 `ws_adapter.js`，无需臃肿老旧的 Socket.io 2.x 客户端库。
- **开箱即用 Docker 支持**：内置多阶段轻量 Dockerfile 与 compose.yaml。

---

## 📁 目录结构

```text
chatroom/rustversion/
├── Cargo.toml                  # Rust 项目依赖管理 (actix-web, actix-ws, tokio, etc.)
├── Dockerfile                  # 多阶段极轻量 Docker 镜像定义
├── compose.yaml                # Docker Compose 一键启动编排
├── views/                      # HTML 视图模板 (index.html, signin.html, signup.html)
├── public/                     # 静态资源 (css, js, doc, ring, images)
│   └── javascripts/
│       └── ws_adapter.js       # 原生 WebSocket 与 chat.js 通信适配器
└── src/
    ├── main.rs                 # 服务主入口、静态目录与路由挂载 (100MB Payload 配置)
    ├── args.rs                 # 命令行参数与环境变量解析 (MAX_UPLOAD_SIZE_MB=100)
    ├── models.rs               # WebSocket Envelope 与业务数据模型
    ├── server.rs               # 聊天房间调度中心 (在线用户管理、广播、历史记录缓存)
    ├── ws.rs                   # WebSocket 连接生命周期与事件处理
    └── routes/                 # HTTP 路由
        ├── mod.rs
        ├── auth.rs             # 页面与登录 Cookie 会话
        └── notice.rs           # /notice 外部广播通知接收接口
```

---

## 🚀 启动与部署

### 1. 本地直接运行
```bash
cargo run
```
浏览器访问：`http://127.0.0.1:28080`

### 2. Docker 容器化运行
```bash
docker compose up -d --build
```
服务将在 `http://0.0.0.0:28080` 启动，上传的文件自动持久化在宿主机 `./uploads` 目录（容器内映射 `/app/public/doc`）。

---

## 📝 开发复盘与避坑总结 (Lessons Learned)

在将 Node.js 聊天室重构为 Rust 的过程中，总结了以下核心技术要点与避坑经验：

### 1. WebSocket 协议适配与兼容老前端
- **痛点**：原有前端 `chat.js` 深度依赖 Socket.io 的 `socket.on(event, cb)` 和 `socket.emit(event, data)`，且各事件（如 `notice`, `system`, `userflush`）传递的参数在 Node.js 端由 `JSON.stringify` 序列化为字符串。
- **解决方案**：在前端编写轻量级 [ws_adapter.js](public/javascripts/ws_adapter.js) 封装原生 WebSocket API，统一定义 `{ type, data }` 消息信封；在客户端派发事件时与原参数格式保持 100% 对齐，后端无需引入臃肿的第三方协议栈即可实现丝滑兼容。

### 2. 大文件 (100MB) 上传限制与 Payload 解锁
- **痛点**：Actix-web 与 WebSocket 默认对传输帧及 HTTP 请求包体有严格大小限制（约 64KB ~ 1MB），直接传输大文件会导致连接被强行切断（`Overflow`）。
- **解决方案**：在 `args.rs` 中引入 `--max-upload-size-mb` 配置（默认 `100`），并在 `main.rs` 中通过 `web::PayloadConfig::new(100 * 1024 * 1024)` 全局解锁 100MB 传输上限。

### 3. 多客户端并发广播与历史消息生命周期
- **痛点**：多用户同时在线时，收发消息存在并发安全问题；新用户加入或页面刷新时需要获取历史记录与 Botty 掉落记录。
- **解决方案**：
  - 采用 Tokio 异步无锁 Channel (`mpsc::UnboundedSender<String>`) 分发消息，保证单客户端阻塞不影响全局广播；
  - 服务端内存缓存最近 30 条公聊记录，针对 Botty 报告采用差异化策略（`Status Report` 仅保留最新 1 条覆盖更新，`Found` 掉落记录全部保留），新用户连接建立后按序自动推送到前端呈现。

### 4. 消除所有权借用悬空 (E0505) 与未声明变量
- **经验**：在路由处理函数中严格定义返回值类型；在向状态容器插入数据转移所有权（`into_inner`）前，提前克隆独立字段（`.to_string()`），确保 Rust 借用检查器（Borrow Checker）零警告通过。
