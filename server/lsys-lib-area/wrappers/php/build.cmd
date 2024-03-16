
@REM "choco install -y visualstudio2019buildtools"
@REM "choco install rustup"



set "crate_dir=%~dp0..\..\"
set "build_dir=%crate_dir%build\"
set "target_dir=%crate_dir%build\target\"


echo off


@REM "build init"
@REM "init 初始化编译环境"


if "%1"=="init" (
	echo "check build env"
	reg query "HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\SxS" >nul 2>&1
	if %ERRORLEVEL% == 0 (
		echo VisualStudio is install
	) else (
		echo please install VisualStudio
		exit 
	)
	where cargo >nul 2>&1
	if %ERRORLEVEL% == 0 (
		echo cargo is install
	) else (
		echo please install cargo
		exit 
	)
	cd %crate_dir%
	mkdir %build_dir%
	mkdir %target_dir%
	echo "init on: %build_dir%,target on:%target_dir%"
	
	echo remove: %build_dir%php-sdk-binary-tools.zip && del %build_dir%php-sdk-binary-tools.zip
	echo remove: %build_dir%php-sdk-binary-tools && rmdir /s /q %build_dir%php-sdk-binary-tools
	echo download php sdk tools start
	powershell -Command "Invoke-WebRequest -Uri 'https://github.com/php/php-sdk-binary-tools/archive/refs/tags/php-sdk-2.2.0.zip' -OutFile '%build_dir%php-sdk-binary-tools.zip'"
	powershell -Command "Expand-Archive -Path '%build_dir%php-sdk-binary-tools.zip' -DestinationPath '%build_dir%'"
	cd %build_dir% && rename php-sdk-binary-tools-php-sdk-2.2.0 php-sdk-binary-tools
	echo download php sdk tools finish
	
	
	echo build lsys_lib_area crate start
	powershell.exe -Command "(Get-Content -Path '%crate_dir%\Cargo.toml') | ForEach-Object { $_ -replace '#\"lib-clib', '\"lib-clib' } | Set-Content -Path '%crate_dir%\Cargo.toml'"
	powershell.exe -Command "(Get-Content -Path '%crate_dir%\Cargo.toml') | ForEach-Object { $_ -replace '#\"data-mysql', '\"data-mysql' } | Set-Content -Path '%crate_dir%\Cargo.toml'"
	cd %crate_dir% && cargo build -r -p lsys-lib-area
	echo build lsys_lib_area crate finish
	
	echo copy lib to ext dir start
	if exist "%crate_dir%target\release\*" (
		echo copy to:%crate_dir%target\release\
		copy /Y %crate_dir%target\release\lsys_lib_area.dll %target_dir%
		copy /Y %crate_dir%target\release\lsys_lib_area.dll %crate_dir%wrappers\php\lib\
		copy /Y %crate_dir%target\release\lsys_lib_area.dll.lib %crate_dir%wrappers\php\lib\lsys_lib_area.lib
		copy /Y %crate_dir%target\release\lsys_lib_area.h %crate_dir%wrappers\php\lib\
	) else (
		echo copy to:%crate_dir%..\target\release\
		copy /Y %crate_dir%..\target\release\lsys_lib_area.dll %target_dir%
		copy /Y %crate_dir%..\target\release\lsys_lib_area.dll %crate_dir%wrappers\php\lib\
		copy /Y %crate_dir%..\target\release\lsys_lib_area.dll.lib %crate_dir%wrappers\php\lib\lsys_lib_area.lib
		copy /Y %crate_dir%..\target\release\lsys_lib_area.h %crate_dir%wrappers\php\lib\
	)
	echo copy lib to ext dir finish
	cd %crate_dir%
)




@REM "build build vs16 x64  8.0 php-8.0.30 "
@REM "build 编译扩展"
@REM "vs16 vs版本"
@REM "x64 arch"
@REM "8.0 扩展使用PHP版本"
@REM "php-8.0.30 本地源码目录"

if "%1"=="build" (
	if exist "%build_dir%%5\..\php-sdk-binary-tools\phpsdk-starter.bat" (
		echo "build tool is download"
	)else (
		echo plase run build.cmd init
		exit 
	)
	if exist "%target_dir%lsys_lib_area.dll" (
		echo archive start
	)else (
		echo please run build.cmd init first,lsys_lib_area.dll not find.
		exit 
	)
	if exist "%build_dir%%5\*" (
		echo build on %build_dir%%5
	) else (
		echo please run build.cmd download first,%build_dir%%5 is not a directory
		exit 
	)
	echo build php ext start
	cd %build_dir%%5
	echo run: ..\php-sdk-binary-tools\phpsdk-starter.bat -c %2 -a %3 -t %crate_dir%\wrappers\php\php_build.bat on: %build_dir%%5
	call ..\php-sdk-binary-tools\phpsdk-starter.bat -c %2 -a %3 -t %crate_dir%\wrappers\php\php_build.bat 
	echo build php ext finish
	echo copy php ext start
	cd %build_dir%%5\%3\Release_TS
	dir .
	cd %build_dir%%5\%3\Release
	dir .
	copy /Y %build_dir%%5\%3\Release_TS\php_lib_area.dll %target_dir%php_lib_area_%4_ts_%2_%3.dll
	copy /Y %build_dir%%5\%3\Release\php_lib_area.dll %target_dir%php_lib_area_%4_nts_%2_%3.dll
	echo copy php ext finish
	cd %crate_dir%

)


@REM "build archive"
@REM "archive 打包编译文件"


if "%1"=="archive" (
	if exist "%target_dir%lsys_lib_area.dll" (
		echo archive start
	)else (
		echo please compile first
		exit 
	)
	echo archive php ext copy start
	copy /Y %crate_dir%LICENSE %target_dir%
	copy /Y %crate_dir%wrappers\php\README.MD %target_dir%
	mkdir %target_dir%example
	copy /Y %crate_dir%wrappers\php\lib_area.api.php %target_dir%example
	copy /Y %crate_dir%wrappers\php\lib_area.stub.php %target_dir%example
	cd %target_dir%
	dir .
	echo archive php ext copy finish
	echo archive php ext zip start
	powershell -Command "Compress-Archive -Path '%target_dir%\*' -DestinationPath %crate_dir%wrappers\php\lsys_lib_area_php_ext.zip"
	echo archive php ext zip finish
	cd %crate_dir%
)

@REM "build clear"
@REM "clear 清理临时文件"

if "%1"=="clear" (
	echo %crate_dir%wrappers\php\lib\lsys_lib_area.dll && del %crate_dir%wrappers\php\lib\lsys_lib_area.dll
	echo %crate_dir%wrappers\php\lib\lsys_lib_area.h && del %crate_dir%wrappers\php\lib\lsys_lib_area.h
	echo %crate_dir%wrappers\php\lib\lsys_lib_area.lib && del %crate_dir%wrappers\php\lib\lsys_lib_area.lib
	rmdir /s /q %build_dir%
	cd %crate_dir% 
)



@REM "build download https://windows.php.net/downloads/releases/php-8.0.30-src.zip php-8.0.30"
@REM "download 下载PHP源文件"
@REM "https://windows.php.net/downloads/releases/php-8.0.30-src.zip 源文件URL"
@REM "php-8.0.30 保存本地目录名"


if "%1"=="download" (
	if exist "%build_dir%\*" (
		echo download save to: %build_dir%
	) else (
		echo please run build.cmd init first
		exit
	)
	cd %build_dir%
	echo download php src %2 start
	echo remove: %build_dir%%3.zip && del %build_dir%%3.zip
	echo remove: %build_dir%%3 && rmdir /s /q %build_dir%%3
	powershell -Command "Invoke-WebRequest -Uri '%2' -OutFile '%build_dir%%3.zip'"
	powershell -Command "Expand-Archive -Path '%build_dir%%3.zip' -DestinationPath '%build_dir%%3'"
	echo %2 | findstr /C:"php-8" >nul 2>&1
	if not errorlevel 1 (
		echo "%build_dir%%3\php*"
		powershell -Command "Get-ChildItem -Path '%build_dir%%3\php*' | Move-Item -Destination '%build_dir%%3-src'"
		rmdir %build_dir%%3
		cd %build_dir%
		rename %3-src %3
		echo move dir finish,build on %build_dir%%3
	) else (
		echo build on %build_dir%%3
	)
	echo download php src %2 finish
	echo copy ext src to src start
	xcopy %crate_dir%wrappers\php %build_dir%%3\ext\lib_area /E /I /Y
	echo copy ext src to src finish
	cd %crate_dir%
)

@REM "先运行 build init 初始化环境"
@REM "build php8"
@REM "php8 编译x64 php8扩展"


if "%1"=="php8" (
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-8.0.30-src.zip php-8.0.30
	cd %crate_dir%wrappers\php && call build.cmd build vs16 x64 8.0 php-8.0.30
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-8.1.27-src.zip php-8.1.27
	cd %crate_dir%wrappers\php && call build.cmd build vs16 x64 8.1 php-8.1.27 
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-8.2.16-src.zip php-8.2.16
	cd %crate_dir%wrappers\php && call build.cmd build vs16 x64 8.2 php-8.2.16
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-8.3.3-src.zip php-8.3.3
	cd %crate_dir%wrappers\php && call build.cmd build vs16 x64 8.3 php-8.3.3
)

@REM "先运行 build init 初始化环境"
@REM "build php7.2_4"
@REM "php7.2_4 编译x64 phpphp7.2至4扩展"

if "%1"=="php7.2_4" (
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-7.4.33-src.zip php-7.4.33
	cd %crate_dir%wrappers\php && call build.cmd build vc15 x64 7.4 php-7.4.33
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/archives/php-7.3.33-src.zip php-7.3.33
	cd %crate_dir%wrappers\php && call build.cmd build vc15 x64 7.3 php-7.3.33
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/archives/php-7.2.9-src.zip php-7.2.9
	cd %crate_dir%wrappers\php && call build.cmd build vc15 x64 7.2 php-7.2.9
	
)

@REM "先运行 build init 初始化环境"
@REM "build php7.1"
@REM "php7.1 编译x64 php7.1扩展"

if "%1"=="php7.1" (
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/archives/php-7.1.33-src.zip php-7.1.33
	cd %crate_dir%wrappers\php && call build.cmd build vc14 x64 7.1 php-7.1.33
)

@REM "build"
@REM "默认编译php8.3扩展及打包成ZIP文件"

if "%1"=="" (
	cd %crate_dir%wrappers\php && call build.cmd init
	cd %crate_dir%wrappers\php && call build.cmd download https://windows.php.net/downloads/releases/php-8.3.3-src.zip php-8.3.3
	cd %crate_dir%wrappers\php && call build.cmd build vs16 x64 8.3 php-8.3.3
	cd %crate_dir%wrappers\php && call build.cmd archive
	cd %crate_dir%wrappers\php && call build.cmd clear
)









