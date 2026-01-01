
@REM 当cargo 中启用 data-sqlite-source 时,下载sqlite源码
@echo off
set "crate_dir=%~dp0"
echo "clear old zip: %crate_dir%sqlite-amalgamation.zip" &&  del %crate_dir%sqlite-amalgamation.zip	
echo "clear old dir: %crate_dir%sqlite-amalgamation" &&  rmdir /s /q %crate_dir%sqlite-amalgamation	
echo "download: https://github.com/shanliu/lsys/releases/download/v0.0.0/sqlite-amalgamation.zip"
powershell -Command "Invoke-WebRequest -Uri 'https://github.com/shanliu/lsys/releases/download/v0.0.0/sqlite-amalgamation.zip' -OutFile '%crate_dir%sqlite-amalgamation.zip'"
echo "unzip sqlite-amalgamation.zip"
powershell -Command "Expand-Archive -Path '%crate_dir%sqlite-amalgamation.zip' -DestinationPath '%crate_dir%sqlite-amalgamation'"
del %crate_dir%sqlite-amalgamation.zip	
echo "download finish"