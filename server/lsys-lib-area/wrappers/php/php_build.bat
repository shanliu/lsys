call .\buildconf.bat && call .\configure.bat --disable-all --enable-cli --enable-lib_area=shared && nmake && call  .\configure.bat --disable-all --enable-cli --disable-zts --enable-lib_area=shared && nmake