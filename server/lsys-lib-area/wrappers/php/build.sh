#!/bin/bash
if command -v cargo >/dev/null 2>&1; then
    echo "cargo is install"
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.profile
fi

script_dir=`pwd`/$(dirname "$0")
extension_dir=`php-config --extension-dir`
echo "source dir:$script_dir" 
echo "install dir:$extension_dir" 

if [ "$(uname)" = "Darwin" ]; then
   sed -i '' "s|#\"lib-clib|\"lib-clib|g" $script_dir/../../Cargo.toml
else
   sed -i "s|#\"lib-clib|\"lib-clib|g" $script_dir/../../Cargo.toml
fi

cd $script_dir/../../ && cargo build -r -p lsys-lib-area \
&& sudo mkdir -p $extension_dir/lib_area \
&& sudo cp $script_dir/../../../target/release/lsys_lib_area.h $extension_dir/lib_area \
&& sudo cp $script_dir/../../../target/release/liblsys_lib_area.so $extension_dir/lib_area \
&& cd $script_dir && phpize --clean && phpize && ./configure --with-lib_area_dir=$extension_dir/lib_area \
&& make && sudo make install
