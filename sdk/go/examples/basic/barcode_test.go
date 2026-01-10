package main

import (
	"bytes"
	"context"
	"fmt"
	"testing"
)

// 二维码创建示例

func TestBarcodeCreate(t *testing.T) {
	sysApi := GetRestApi()

	// 创建二维码
	result, err := sysApi.BarcodeCreate(context.Background(), "https://www.lsys.cc", 2)
	if err == nil {
		fmt.Printf("barcode type: %s\n", result.Type)
		fmt.Printf("barcode data length: %d\n", len(result.Data))
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// 二维码解析示例
// 注意: 解析需要上传图片文件
func TestBarcodeParse(t *testing.T) {
	sysApi := GetRestApi()

	// 示例图片数据 (实际使用时需要读取真实图片文件)
	// file, err := os.Open("./test_qrcode.png")
	// if err != nil {
	//     t.Fatal(err)
	// }
	// defer file.Close()

	// 模拟空图片数据，使用 bytes.Reader 作为 io.Reader
	fileData := []byte{}
	fileReader := bytes.NewReader(fileData)
	result, err := sysApi.BarcodeParse(context.Background(), fileReader, "test.png", true)
	if err == nil {
		for _, record := range result.Record {
			fmt.Printf("barcode text: %s\n", record.Data.Text)
			fmt.Printf("barcode type: %s\n", record.Data.Type)
			fmt.Printf("barcode hash: %s\n", record.Data.Hash)
		}
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}
