--TEST--
LibArea test
--SKIPIF--
<?php
if (!extension_loaded('lib_area')) {
    echo 'skip';
}
?>
--FILE--
<?php
use LsExt\LibArea;
LibArea::initCsv(__DIR__."/../../../data/2023-7-area-code.csv.gz",__DIR__."/../../../data/2023-7-area-geo.csv.gz",sys_get_temp_dir()."/lib_area");
$ret =LibArea::codeSearch("guangdong");
echo count($ret);
$ret =LibArea::codeSearch("ddddddddddddddddd");
echo count($ret);
$ret =LibArea::codeFind("4414");
echo count($ret);
$ret =LibArea::codeFind("121212121212");
echo count($ret);
$ret =LibArea::codeRelated("4414");
echo count($ret);
$ret =LibArea::codeRelated("121212121212");
echo count($ret);
$ret =LibArea::geoSearch(22.57729, 113.89409);
echo count($ret);
$ret =LibArea::codeChilds("");
echo count($ret);
$ret =LibArea::codeChilds("1212121212121");
echo count($ret);
LibArea::geoReload();
LibArea::codeReload();
$ret =LibArea::codeSearch("guangdong");
echo count($ret);
$ret =LibArea::codeSearch("ddddddddddddddddd");
echo count($ret);
$ret =LibArea::codeFind("4414");
echo count($ret);
$ret =LibArea::codeFind("121212121212");
echo count($ret);
$ret =LibArea::codeRelated("4414");
echo count($ret);
$ret =LibArea::codeRelated("121212121212");
echo count($ret);
$ret =LibArea::geoSearch(22.57729, 113.89409);
echo count($ret);
$ret =LibArea::codeChilds("");
echo count($ret);
$ret =LibArea::codeChilds("1212121212121");
echo count($ret);
LibArea::shutdown();
?>
--EXPECT--
1002136334010021363340
