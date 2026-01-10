package lsyslib

import (
	"context"
	"io"
	"net/http"
	"rest_client"
	"time"
)

const (
	BarcodeCreate = 700
	BarcodeParse  = 701
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		BarcodeCreate: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/barcode",
			Method:     "create",
			Timeout:    60 * time.Second,
		},
		BarcodeParse: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/barcode",
			Method:     "parse",
			Timeout:    60 * time.Second,
		},
	})
}

// BarcodeCreateResult 二维码创建结果
type BarcodeCreateResult struct {
	Data string `json:"data"` // 图片Base64数据
	Type string `json:"type"` // 图片类型
}

// BarcodeCreate 创建二维码
// contents 生成二维码内容
// codeId 二维码应用ID
func (receiver *LsysApi) BarcodeCreate(ctx context.Context, contents string, codeId int) (*BarcodeCreateResult, error) {
	data1 := (<-receiver.rest.Do(ctx, BarcodeCreate, map[string]interface{}{
		"contents": contents,
		"code_id":  codeId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return &BarcodeCreateResult{
		Data: data1.GetData("response.data").String(),
		Type: data1.GetData("response.type").String(),
	}, nil
}

// BarcodePosition 二维码位置
type BarcodePosition struct {
	X string `json:"x"`
	Y string `json:"y"`
}

// BarcodeParseData 二维码解析数据
type BarcodeParseData struct {
	Hash     string            `json:"hash"`
	Text     string            `json:"text"`
	Type     string            `json:"type"`
	Position []BarcodePosition `json:"position"`
}

// BarcodeParseRecord 二维码解析记录
type BarcodeParseRecord struct {
	Status string           `json:"status"`
	Data   BarcodeParseData `json:"data"`
}

// BarcodeParseResult 二维码解析结果
type BarcodeParseResult struct {
	Record []BarcodeParseRecord `json:"record"`
}

// BarcodeParse 解析二维码
// fileReader 图片文件读取器，可以是 *os.File 或其他 io.Reader
// fileName 文件名
// tryHarder 是否更仔细地解析
func (receiver *LsysApi) BarcodeParse(ctx context.Context, fileReader io.Reader, fileName string, tryHarder bool) (*BarcodeParseResult, error) {
	// 使用 MultipartParam 发送请求，直接传递文件读取器
	data1 := (<-receiver.rest.Do(ctx, BarcodeParse, &MultipartParam{
		Payload: map[string]interface{}{
			"try_harder": tryHarder,
		},
		Files: []MultipartFile{
			{
				FieldName: "file",
				FileName:  fileName,
				Reader:    fileReader,
			},
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}

	var result BarcodeParseResult
	for _, item := range data1.GetData("response.record").Array() {
		var positions []BarcodePosition
		for _, pos := range item.Get("data.position").Array() {
			positions = append(positions, BarcodePosition{
				X: pos.Get("x").String(),
				Y: pos.Get("y").String(),
			})
		}
		result.Record = append(result.Record, BarcodeParseRecord{
			Status: item.Get("status").String(),
			Data: BarcodeParseData{
				Hash:     item.Get("data.hash").String(),
				Text:     item.Get("data.text").String(),
				Type:     item.Get("data.type").String(),
				Position: positions,
			},
		})
	}
	return &result, nil
}
