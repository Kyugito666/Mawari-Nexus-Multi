@echo off
title Orchestrator Monitor
color 0A

:loop
cls
echo ╔════════════════════════════════════════════════╗
echo ║       ORCHESTRATOR MONITOR                    ║
echo ╚════════════════════════════════════════════════╝
echo.
echo Waktu: %date% %time%
echo.

cd /d PATH

echo ─────────────────────────────────────────────────
echo STATUS ORKESTRATOR (state.json):
echo ─────────────────────────────────────────────────
if exist state.json (
    type state.json
) else (
    echo Tidak ada file state.json ditemukan.
)

echo.
echo ─────────────────────────────────────────────────
echo CODESPACES AKTIF (via gh cs list):
echo ─────────────────────────────────────────────────
gh cs list

echo.
echo ─────────────────────────────────────────────────
echo Refresh dalam 30 detik... (Tekan Ctrl+C untuk berhenti)
timeout /t 30 /nobreak >nul
goto loop
