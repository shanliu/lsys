mkdir build
xcopy "README.MD" build /Y
xcopy LICENSE build /Y

mkdir server\\examples\\lsys-actix-web\data
cd server\\examples\\lsys-actix-web 
cargo version
rustc -V
cargo update
cargo build -r -vvv 
cd ..\\..\\..

xcopy /E /I /Y server\\assets build\\assets
xcopy server\\target\\release\\lsys-actix-web.exe build /Y 
xcopy /E /I /Y server\\examples\\lsys-actix-web\\config build\\config
xcopy /E /I /Y server\\examples\\lsys-actix-web\\data build\\data
xcopy /E /I /Y server\\examples\\lsys-actix-web\\locale build\\locale
xcopy /E /I /Y server\\examples\\lsys-actix-web\\static build\\static
xcopy server\\examples\\lsys-actix-web\\.env build /Y

cd ui\\web 
call npm install
call npm run build
cd ..\\.. 
mkdir ui\\public
cd ui\\wap 
call npm install
call npm run build
cd ..\\.. 
mkdir ui\\public
xcopy /E /I /Y ui\\public build\\ui


powershell.exe -Command "(Get-Content -Path 'build\\config\\app.toml') | ForEach-Object { $_ -replace '../../../ui/public/', './ui/' } | Set-Content -Path 'build\\config\\app.toml'"

cd build 
powershell.exe -Command "Compress-Archive -Path './' -DestinationPath '../lsys.zip'"
cd ..

echo The compilation was successful, please enable the service with ( ./lsys-actix-web ) to the service and access it via http://127.0.0.1:8080
