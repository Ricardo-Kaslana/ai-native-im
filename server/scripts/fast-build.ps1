# Fast Build Script for Windows
# 快速编译脚本 - Windows PowerShell
#
# 使用方法:
#   .\scripts\fast-build.ps1           # 快速检查
#   .\scripts\fast-build.ps1 build     # 快速构建
#   .\scripts\fast-build.ps1 test      # 快速测试
#   .\scripts\fast-build.ps1 clean     # 清理缓存

param(
    [Parameter(Position=0)]
    [ValidateSet("check", "build", "test", "clean", "release")]
    [string]$Command = "check"
)

# 设置环境变量优化编译
$env:CARGO_INCREMENTAL = "1"
$env:RUSTFLAGS = "-C debuginfo=1"

# 获取 CPU 核心数
$cores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors

Write-Host "🚀 Fast Build - Using $cores CPU cores" -ForegroundColor Cyan
Write-Host "Command: $Command" -ForegroundColor Yellow

$startTime = Get-Date

switch ($Command) {
    "check" {
        Write-Host "Running cargo check..." -ForegroundColor Green
        cargo check --jobs $cores
    }
    "build" {
        Write-Host "Running cargo build..." -ForegroundColor Green
        cargo build --jobs $cores
    }
    "test" {
        Write-Host "Running cargo test (lib only)..." -ForegroundColor Green
        # 只运行库测试，跳过集成测试
        cargo test --lib --jobs $cores -- --test-threads=$cores
    }
    "clean" {
        Write-Host "Cleaning incremental cache..." -ForegroundColor Green
        # 只清理增量编译缓存，保留依赖
        if (Test-Path "target\debug\incremental") {
            Remove-Item -Recurse -Force "target\debug\incremental"
        }
        if (Test-Path "target\debug\.fingerprint\ai-im-server*") {
            Remove-Item -Recurse -Force "target\debug\.fingerprint\ai-im-server*"
        }
        Write-Host "Cache cleaned. Dependencies preserved." -ForegroundColor Green
    }
    "release" {
        Write-Host "Running release build..." -ForegroundColor Green
        cargo build --release --jobs $cores
    }
}

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "`n✅ Completed in $($duration.TotalSeconds.ToString('F2')) seconds" -ForegroundColor Cyan
