<#
.SYNOPSIS
    Chatroom 跨平台服务发布打包脚本 (Windows PowerShell 版)
.DESCRIPTION
    使用 cross 交叉编译为纯静态 musl 二进制，并将程序、静态资源和一键安装脚本打包为 .tar.gz 发布包。
.PARAMETER Target
    编译目标架构，默认: x86_64-unknown-linux-musl
    常见选项:
      - x86_64-unknown-linux-musl     (x86_64 软路由 / Linux 服务器)
      - aarch64-unknown-linux-musl    (Cortex-A 64位，如树莓派、RK3568、MT7981 OpenWrt)
      - armv7-unknown-linux-musleabihf (Cortex-A 32位，如早期 ARMv7 路由器)
#>
param(
    [string]$Target = "x86_64-unknown-linux-musl"
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "🚀 开始构建并打包 Chatroom 服务程序..." -ForegroundColor Cyan
Write-Host "🎯 目标平台 Target: $Target" -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan

# 1. 检查 cross 工具链
if (-not (Get-Command "cross" -ErrorAction SilentlyContinue)) {
    Write-Warning "未检测到 cross 工具链！"
    Write-Host "请先运行以下命令安装 cross (需确保本机已安装 Docker 或 Podman):" -ForegroundColor Green
    Write-Host "  cargo install cross --git https://github.com/cross-rs/cross" -ForegroundColor White
    exit 1
}

# 2. 读取项目版本号
$CargoToml = Get-Content "Cargo.toml" -Raw
if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
    $Version = $matches[1]
} else {
    $Version = "0.1.0"
}
Write-Host "ℹ 项目版本: v$Version" -ForegroundColor Gray

# 3. 执行交叉编译
Write-Host "`n📦 [1/3] 正在通过 cross 进行交叉编译 (Release 模式)..." -ForegroundColor Green
cross build --target $Target --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "编译失败，请检查 Docker 服务是否已开启或 target 参数是否正确。"
    exit 1
}

# 4. 组装发布包目录
$DistRoot = "dist"
$PkgName = "chatroom-v${Version}-${Target}"
$PkgDir = Join-Path $DistRoot $PkgName

Write-Host "`n📁 [2/3] 组装发布目录: $PkgDir ..." -ForegroundColor Green
if (Test-Path $PkgDir) {
    Remove-Item -Recurse -Force $PkgDir
}
New-Item -ItemType Directory -Force -Path $PkgDir | Out-Null

$BinarySource = "target/$Target/release/chatroom"
if (-not (Test-Path $BinarySource)) {
    Write-Error "找不到编译产物: $BinarySource"
    exit 1
}

Copy-Item $BinarySource -Destination "$PkgDir/chatroom"
Copy-Item -Recurse "public" -Destination "$PkgDir/public"
Copy-Item -Recurse "views" -Destination "$PkgDir/views"
Copy-Item "scripts/install.sh" -Destination "$PkgDir/install.sh"

# 5. 打包为 tar.gz
Write-Host "`n🗜️ [3/3] 正在压缩打包为 .tar.gz ..." -ForegroundColor Green
$TarGzPath = "dist/${PkgName}.tar.gz"
if (Test-Path $TarGzPath) {
    Remove-Item -Force $TarGzPath
}

# 使用 tar 打包 (Windows 10/11 内置 bsdtar)
tar -czvf $TarGzPath -C $DistRoot $PkgName

# 清理临时中间目录
Remove-Item -Recurse -Force $PkgDir

Write-Host "`n============================================================" -ForegroundColor Cyan
Write-Host "🎉 发布包打包完成！" -ForegroundColor Green
Write-Host "📦 最终发布文件: $TarGzPath" -ForegroundColor White
Write-Host "------------------------------------------------------------" -ForegroundColor Gray
Write-Host "💡 目标机器 (OpenWrt / Linux) 部署步骤:" -ForegroundColor Yellow
Write-Host "  1. 上传 $TarGzPath 到目标机器"
Write-Host "  2. 执行解压: tar -zxvf ${PkgName}.tar.gz"
Write-Host "  3. 进入目录并一键安装服务: cd ${PkgName} && sudo ./install.sh"
Write-Host "============================================================" -ForegroundColor Cyan
