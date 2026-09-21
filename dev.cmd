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

REM Don tien trinh mo coi tu phien truoc.
REM
REM Hai loi nay lap lai nhieu lan trong du an:
REM   1. "Port 1420 is already in use" — vite dev server cua phien truoc con
REM      song sau khi cua so terminal bi dong (Ctrl+C khong luon dung duoc
REM      tien trinh con tren Windows).
REM   2. "LNK1104: cannot open file git-plum.exe" / "Access is denied
REM      (os error 5)" — tien trinh git-plum.exe va cac tien trinh con
REM      msedgewebview2.exe con giu handle vao file exe nen cargo khong ghi
REM      lai duoc.
REM
REM Chi tat tien trinh nghe tren dung port 1420 (khong tat moi node.exe —
REM may co the dang chay viec khac).

echo Dang don tien trinh mo coi tu phien truoc...

for /f "tokens=5" %%p in ('netstat -ano ^| findstr /r /c:":1420 .*LISTENING"') do (
    echo   - tat tien trinh dang giu port 1420 ^(PID %%p^)
    taskkill /f /pid %%p >nul 2>&1
)

REM Tat git-plum.exe truoc: WebView2 la tien trinh con cua no, nen tat cha
REM thi phan lon tien trinh con tu thoat theo.
taskkill /f /im git-plum.exe >nul 2>&1
if not errorlevel 1 echo   - tat git-plum.exe con sot lai

REM WebView2 con sot lai thi chi tat dung cai la con cua git-plum.exe.
REM KHONG dung `taskkill /im msedgewebview2.exe` — lenh do tat MOI tien trinh
REM WebView2 tren may, ke ca cua ung dung Tauri/Electron khac dang mo.
powershell -NoProfile -Command ^
  "$w = Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" -ErrorAction SilentlyContinue; if ($w) { $plum = @(Get-CimInstance Win32_Process -Filter \"Name='git-plum.exe'\" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty ProcessId); $orphans = $w | Where-Object { $_.ParentProcessId -in $plum -or -not (Get-Process -Id $_.ParentProcessId -ErrorAction SilentlyContinue) }; if ($orphans) { $orphans | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }; Write-Host ('   - tat ' + @($orphans).Count + ' tien trinh WebView2 mo coi') } }" 2>nul

echo.
echo Dang khoi chay git-plum...
echo Lan dau bien dich Rust mat vai phut, cac lan sau nhanh hon.
echo.

call npm run tauri:dev

endlocal
