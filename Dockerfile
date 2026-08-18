# ----- 第一阶段：基于 Rust 官方镜像多阶段构建编译 -----
FROM rust:1-slim-bookworm as build

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

# ----- 第二阶段：采用轻量级运行镜像组装 -----
FROM bitnami/minideb:latest

WORKDIR /app

RUN mkdir -p /usr/share/zoneinfo public/doc data

# 复制时区与证书
COPY --from=build \
  /usr/share/zoneinfo \
  /usr/share/zoneinfo

COPY --from=build \
  /etc/ssl/certs/ca-certificates.crt \
  /etc/ssl/certs/ca-certificates.crt

# 复制编译好的 Rust 可执行二进制文件
COPY --from=build \
  /app/target/release/chatroom \
  /usr/bin/chatroom

# 复制前端静态资源与页面
COPY public ./public
COPY views ./views

# 暴露端口
EXPOSE 28080

# 启动命令
ENTRYPOINT ["chatroom"]
