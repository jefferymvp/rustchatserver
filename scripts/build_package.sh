#!/usr/bin/env bash
set -e

# ========================================================
# Chatroom 跨平台服务发布打包脚本 (Linux / macOS / CI 版)
# ========================================================

TARGET="${1:-x86_64-unknown-linux-musl}"

echo "============================================================"
echo "🚀 开始构建并打包 Chatroom 服务程序..."
echo "🎯 目标平台 Target: ${TARGET}"
echo "============================================================"

# 检查 cross 或 cargo
BUILD_CMD="cargo"
if command -v cross >/dev/null 2>&1; then
    BUILD_CMD="cross"
    echo "✔ 使用 cross 交叉编译工具链"
else
    echo "ℹ 未找到 cross，尝试使用本地 cargo 工具链"
fi

# 获取版本号
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2 || echo "0.1.0")
echo "ℹ 项目版本: v${VERSION}"

# 1. 编译
echo ""
echo "📦 [1/3] 正在编译产物 (${BUILD_CMD} build --target ${TARGET} --release)..."
${BUILD_CMD} build --target "${TARGET}" --release

# 2. 组装临时发布目录
PKG_NAME="chatroom-v${VERSION}-${TARGET}"
DIST_ROOT="dist"
PKG_DIR="${DIST_ROOT}/${PKG_NAME}"

echo ""
echo "📁 [2/3] 组装发布目录: ${PKG_DIR} ..."
rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}"

BIN_SRC="target/${TARGET}/release/chatroom"
if [ ! -f "${BIN_SRC}" ]; then
    echo "❌ 找不到编译产物: ${BIN_SRC}"
    exit 1
fi

cp -f "${BIN_SRC}" "${PKG_DIR}/chatroom"
chmod +x "${PKG_DIR}/chatroom"
cp -rf public "${PKG_DIR}/"
cp -rf views "${PKG_DIR}/"
cp -f scripts/install.sh "${PKG_DIR}/install.sh"
chmod +x "${PKG_DIR}/install.sh"

# 3. 打包为 tar.gz
echo ""
echo "🗜️ [3/3] 正在压缩打包为 .tar.gz ..."
TAR_GZ_PATH="${DIST_ROOT}/${PKG_NAME}.tar.gz"
rm -f "${TAR_GZ_PATH}"

tar -czvf "${TAR_GZ_PATH}" -C "${DIST_ROOT}" "${PKG_NAME}"
rm -rf "${PKG_DIR}"

echo ""
echo "============================================================"
echo "🎉 发布包打包完成！"
echo "📦 产物路径: ${TAR_GZ_PATH}"
echo "------------------------------------------------------------"
echo "💡 部署使用方法:"
echo "  1. 复制 ${TAR_GZ_PATH} 至 OpenWrt 或 Linux 目标机器"
echo "  2. tar -zxvf ${PKG_NAME}.tar.gz"
echo "  3. cd ${PKG_NAME} && sudo ./install.sh"
echo "============================================================"
