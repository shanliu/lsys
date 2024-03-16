--TEST--
Check if lib_area is loaded
--SKIPIF--
<?php
if (!extension_loaded('lib_area')) {
    echo 'skip';
}
?>
--FILE--
<?php
echo 'The extension "lib_area" is available';
?>
--EXPECT--
The extension "lib_area" is available
