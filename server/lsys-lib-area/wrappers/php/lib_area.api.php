<?php

//1.配置数据文件路径
if (is_dir(__DIR__ . "/data")){
    define("CODE_DATA_PATH", __DIR__ . "/data/2023-7-area-code.csv.gz") ;
	define("GEO_DATA_PATH",__DIR__ . "/data/2023-7-area-geo.csv.gz") ;
}else{
	define("CODE_DATA_PATH", __DIR__ . "/../../data/2023-7-area-code.csv.gz") ;
	define("GEO_DATA_PATH",__DIR__ . "/../../data/2023-7-area-geo.csv.gz") ;
}

//2. 测试命令
//php -S localhost:8000
//curl 'http://localhost:8000/lib_area.api.php?code=4414'
//curl 'http://localhost:8000/lib_area.api.php?action=code&code=4414'
//curl 'http://localhost:8000/lib_area.api.php?key_word=guangdong&action=search'
//curl 'http://localhost:8000/lib_area.api.php?action=related&code=4414'
//curl 'http://localhost:8000/lib_area.api.php?action=geo&lat=26.61474&lng=114.13548'
use LsExt\LibArea;

try {
    LibArea::initCsv(CODE_DATA_PATH, GEO_DATA_PATH, sys_get_temp_dir()."/lib_area");
    $out = array('status' => true, 'msg' => 'ok', 'data' => action($_GET));
} catch (Exception $e) {
    $out = array('status' => FALSE, 'msg' => $e->getMessage());
}
echo json_encode($out, JSON_UNESCAPED_UNICODE);

//--------------------- action -------------------------------
function action(array $param)
{
    switch ($param['action'] ?? '') {
        case 'list':
        case '':
            return LibArea::codeChilds($param['code'] ?? '');
        case 'search':
            if (!empty($param['key_word'])) {
                return LibArea::codeSearch($param['key_word']);
            } else {
                return LibArea::codeChilds("");
            }
        case 'code':
            return LibArea::codeFind($param['code'] ?? '');
        case 'geo':
            return LibArea::geoSearch(floatval($param['lat'] ?? '0'), floatval($param['lng'] ?? '0'));
        case 'related':
            return LibArea::codeRelated($param['code'] ?? '');
        case 'reload':
            LibArea::geoReload();
            LibArea::codeReload();
            return [];
        default:
            throw new ErrorException("bad action");
    }
}
