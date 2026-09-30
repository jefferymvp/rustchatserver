#!/bin/sh
set -e

# ========================================================
# Chatroom 聊天室服务通用一键安装 / 卸载管理脚本
# 支持: OpenWrt (procd) 与 常规 Linux (systemd)
# ========================================================

APP_NAME="chatroom"
INSTALL_DIR="/usr/share/${APP_NAME}"
BIN_PATH="${INSTALL_DIR}/${APP_NAME}"
DEFAULT_PORT=28080
DEFAULT_BIND="0.0.0.0"

PORT="${PORT:-$DEFAULT_PORT}"
BIND="${BIND:-$DEFAULT_BIND}"
DATA_DIR="${DATA_DIR:-${INSTALL_DIR}/data}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() {
    printf "${GREEN}✔ %s${NC}\n" "$1"
}

log_warn() {
    printf "${YELLOW}⚠ %s${NC}\n" "$1"
}

log_err() {
    printf "${RED}✘ %s${NC}\n" "$1"
}

check_root() {
    if [ "$(id -u)" != "0" ]; then
        log_err "请使用 root 权限执行此脚本 (例如: sudo ./install.sh)"
        exit 1
    fi
}

detect_init_system() {
    if [ -f "/etc/openwrt_release" ]; then
        echo "openwrt"
    elif command -v systemctl >/dev/null 2>&1; then
        echo "systemd"
    else
        echo "unknown"
    fi
}

do_install() {
    check_root

    echo "=========================================================="
    printf "${CYAN}🚀 开始安装 %s 实时聊天室服务...${NC}\n" "${APP_NAME}"
    echo "=========================================================="

    # 获取当前安装包目录
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

    if [ ! -f "${SCRIPT_DIR}/${APP_NAME}" ]; then
        log_err "未在当前目录找到二进制执行文件: ${SCRIPT_DIR}/${APP_NAME}"
        exit 1
    fi

    # 1. 创建目标目录
    log_info "创建程序部署目录: ${INSTALL_DIR}"
    mkdir -p "${INSTALL_DIR}"
    mkdir -p "${DATA_DIR}"

    # 2. 复制二进制可执行文件
    log_info "部署二进制可执行文件..."
    cp -f "${SCRIPT_DIR}/${APP_NAME}" "${BIN_PATH}"
    chmod +x "${BIN_PATH}"

    # 3. 复制前端静态文件与模板
    if [ -d "${SCRIPT_DIR}/public" ]; then
        log_info "部署静态资源文件 public/ ..."
        cp -rf "${SCRIPT_DIR}/public" "${INSTALL_DIR}/"
    else
        log_warn "未发现 public 目录，若已通过 rust-embed 内嵌则忽略。"
    fi

    if [ -d "${SCRIPT_DIR}/views" ]; then
        log_info "部署页面视图文件 views/ ..."
        cp -rf "${SCRIPT_DIR}/views" "${INSTALL_DIR}/"
    else
        log_warn "未发现 views 目录，若已通过 rust-embed 内嵌则忽略。"
    fi

    # 确保上传目录存在
    mkdir -p "${INSTALL_DIR}/public/doc"

    # 4. 判断并生成守护服务
    INIT_SYS="$(detect_init_system)"

    if [ "$INIT_SYS" = "openwrt" ]; then
        log_info "检测到系统环境: OpenWrt (采用 procd 服务管理)"
        
        SERVICE_FILE="/etc/init.d/${APP_NAME}"
        cat << EOF > "${SERVICE_FILE}"
#!/bin/sh /etc/rc.common

START=95
STOP=10
USE_PROCD=1
PROG=${BIN_PATH}
WORKDIR=${INSTALL_DIR}

start_service() {
    procd_open_instance
    procd_set_param respawn 3600 5 0
    procd_set_param user root
    procd_set_param command \$PROG
    procd_append_param command --port ${PORT}
    procd_append_param command --bind ${BIND}
    procd_append_param command --data-dir ${DATA_DIR}
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
}
EOF
        chmod +x "${SERVICE_FILE}"

        log_info "注册开机自启..."
        "${SERVICE_FILE}" enable

        log_info "启动服务..."
        "${SERVICE_FILE}" restart

    elif [ "$INIT_SYS" = "systemd" ]; then
        log_info "检测到系统环境: Linux (采用 systemd 服务管理)"

        SYSTEMD_FILE="/etc/systemd/system/${APP_NAME}.service"
        cat << EOF > "${SYSTEMD_FILE}"
[Unit]
Description=Rust High-Performance Chatroom Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=${INSTALL_DIR}
ExecStart=${BIN_PATH} --port ${PORT} --bind ${BIND} --data-dir ${DATA_DIR}
Restart=always
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

        log_info "重载 systemd 守护进程..."
        systemctl daemon-reload

        log_info "设置服务开机自启并启动..."
        systemctl enable "${APP_NAME}"
        systemctl restart "${APP_NAME}"

    else
        log_warn "未检测到已知的服务管理器 (既非 OpenWrt 也非 systemd)。"
        log_warn "你可以进入 ${INSTALL_DIR} 后手动启动: ./chatroom --port ${PORT} --bind ${BIND}"
    fi

    echo "=========================================================="
    log_info "🎉 ${APP_NAME} 服务已成功部署并启动！"
    printf "ℹ 监听地址: http://%s:%s\n" "${BIND}" "${PORT}"
    printf "ℹ 安装目录: %s\n" "${INSTALL_DIR}"
    printf "ℹ 数据持久化目录: %s\n" "${DATA_DIR}"
    printf "ℹ 查看状态或重启: ./install.sh status | restart\n"
    printf "ℹ 卸载服务: ./install.sh uninstall\n"
    echo "=========================================================="
}

do_uninstall() {
    check_root
    echo "=========================================================="
    printf "${YELLOW}🗑️ 正在停止并卸载 %s 服务...${NC}\n" "${APP_NAME}"
    echo "=========================================================="

    INIT_SYS="$(detect_init_system)"

    if [ "$INIT_SYS" = "openwrt" ]; then
        SERVICE_FILE="/etc/init.d/${APP_NAME}"
        if [ -f "${SERVICE_FILE}" ]; then
            "${SERVICE_FILE}" stop || true
            "${SERVICE_FILE}" disable || true
            rm -f "${SERVICE_FILE}"
            log_info "已移除 OpenWrt 服务脚本: ${SERVICE_FILE}"
        fi
    elif [ "$INIT_SYS" = "systemd" ]; then
        SYSTEMD_FILE="/etc/systemd/system/${APP_NAME}.service"
        if [ -f "${SYSTEMD_FILE}" ]; then
            systemctl stop "${APP_NAME}" || true
            systemctl disable "${APP_NAME}" || true
            rm -f "${SYSTEMD_FILE}"
            systemctl daemon-reload
            log_info "已移除 systemd 服务单元: ${SYSTEMD_FILE}"
        fi
    fi

    if [ -d "${INSTALL_DIR}" ]; then
        rm -rf "${INSTALL_DIR}"
        log_info "已清理安装目录: ${INSTALL_DIR}"
    fi

    log_info "✔ 卸载完成！"
}

do_status() {
    INIT_SYS="$(detect_init_system)"
    if [ "$INIT_SYS" = "openwrt" ]; then
        /etc/init.d/${APP_NAME} status || true
        echo "提示: 查看 OpenWrt 实时运行日志可执行: logread -e ${APP_NAME} -f"
    elif [ "$INIT_SYS" = "systemd" ]; then
        systemctl status "${APP_NAME}" --no-pager
    else
        ps | grep "${APP_NAME}" | grep -v grep || true
    fi
}

do_restart() {
    INIT_SYS="$(detect_init_system)"
    if [ "$INIT_SYS" = "openwrt" ]; then
        /etc/init.d/${APP_NAME} restart
    elif [ "$INIT_SYS" = "systemd" ]; then
        systemctl restart "${APP_NAME}"
    fi
    log_info "已尝试重启 ${APP_NAME} 服务。"
}

do_stop() {
    INIT_SYS="$(detect_init_system)"
    if [ "$INIT_SYS" = "openwrt" ]; then
        /etc/init.d/${APP_NAME} stop
    elif [ "$INIT_SYS" = "systemd" ]; then
        systemctl stop "${APP_NAME}"
    fi
    log_info "已停止 ${APP_NAME} 服务。"
}

case "$1" in
    uninstall)
        do_uninstall
        ;;
    status)
        do_status
        ;;
    restart)
        do_restart
        ;;
    stop)
        do_stop
        ;;
    *)
        do_install
        ;;
esac
