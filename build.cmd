
set "root_dir=%~dp0\"
set "build_dir=%root_dir%build\"
mkdir %build_dir%
xcopy "README.MD" %build_dir% /Y
xcopy LICENSE %build_dir% /Y

mkdir %root_dir%server\\examples\\lsys-actix-web\data
cd %root_dir%server\\examples\\lsys-actix-web 
cargo version
rustc -V
cargo update
cargo build -r -vvv 
cd %root_dir%server && call sql_merge.cmd

mkdir %build_dir%assets
xcopy %root_dir%server\\tables.sql %build_dir%assets\\ /Y
xcopy %root_dir%server\\target\\release\\lsys-actix-web.exe %build_dir% /Y 
xcopy /E /I /Y %root_dir%server\\examples\\lsys-actix-web\\config %build_dir%config
xcopy /E /I /Y %root_dir%server\\examples\\lsys-actix-web\\data %build_dir%data
xcopy /E /I /Y %root_dir%server\\examples\\lsys-actix-web\\locale %build_dir%locale
xcopy /E /I /Y %root_dir%server\\examples\\lsys-actix-web\\static %build_dir%static
xcopy %root_dir%server\\examples\\lsys-actix-web\\.env %build_dir% /Y
mkdir %build_dir%logs

mkdir %root_dir%ui\\dist
cd %root_dir%ui
call npm install -g pnpm
call pnpm install
call pnpm run build
cd %root_dir%docs
call pnpm install
call pnpm run docs:build
cd %root_dir%
xcopy /E /I /Y %root_dir%ui\\dist %build_dir%ui


powershell.exe -Command "(Get-Content -Path '%build_dir%config\\app.toml') | ForEach-Object { $_ -replace '../../../ui/dist/', './ui/' } | Set-Content -Path '%build_dir%config\\app.toml'"


set has_assets=false
set has_tar=false

:loop
if "%1"=="" goto endloop

if "%1"=="assets" (
  set has_assets=true
)

if "%1"=="zip" (
  set has_tar=true
)

shift
goto loop

:endloop

if "%has_assets%"=="true" (
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-code.csv.gz' -OutFile '%build_dir%data\2023-7-area-code.csv.gz'"
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-geo.csv.gz' -OutFile '%build_dir%data\2023-7-area-geo.csv.gz'"
    powershell -Command "Invoke-WebRequest -Uri 'https://github.com/shanliu/lsys/releases/download/v0.0.0/IP2LOCATION-LITE-DB11.BIN.zip' -OutFile '%build_dir%data\IP2LOCATION-LITE-DB11.BIN.zip'"
    powershell -Command "Expand-Archive -Path '%build_dir%data\IP2LOCATION-LITE-DB11.BIN.zip' -DestinationPath '%build_dir%data'"
    del %build_dir%data\IP2LOCATION-LITE-DB11.BIN.zip
)

if "%has_tar%"=="true" (
    powershell.exe -Command "Compress-Archive -Path '%build_dir%' -DestinationPath '%build_dir%..\\lsys-for-windows.zip'"
)

cd %root_dir%

echo off
echo "The compilation was successful, please enable the service with ( cd build &&  .\lsys-actix-web ) to the service and access it via http://127.0.0.1:8080"
