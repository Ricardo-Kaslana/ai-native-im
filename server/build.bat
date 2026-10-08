@echo off
REM Build script for AI-Native IM server
REM This script ensures PROTOC is set correctly before building

REM Set PROTOC environment variable
set PROTOC=.\protoc.exe

REM Build the project
cargo build %*