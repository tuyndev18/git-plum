@echo off
REM Khởi chạy git-plum ở chế độ phát triển.
REM
REM Vì sao cần tệp này: rustup ghi %USERPROFILE%\.cargo\bin vào PATH của
REM Windows khi cài, nhưng mọi tiến trình đang chạy từ trước đó — cửa sổ dòng
REM lệnh, VS Code, Explorer — vẫn giữ bản PATH cũ cho tới khi được khởi động
REM lại. Tệp này thêm thẳng đường dẫn đó vào PATH của phiên hiện tại nên chạy
REM được ngay, không cần khởi động lại máy.
REM
REM Sau khi khởi động lại Windows một lần, chạy thẳng `npm run tauri:dev`
REM cũng được, không cần tệp này nữa.

setlocal

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where cargo >nul 2>&1
if errorlevel 1 (
    echo.
    echo Khong tim thay cargo tai "%USERPROFILE%\.cargo\bin".
    echo Kiem tra Rust da cai chua:
    echo     "%USERPROFILE%\.cargo\bin\rustup.exe" toolchain list
    echo.
    exit /b 1
)

cd /d "%~dp0"

echo Dang khoi chay git-plum...
echo Lan dau bien dich Rust mat vai phut, cac lan sau nhanh hon.
echo.

call npm run tauri:dev

endlocal
