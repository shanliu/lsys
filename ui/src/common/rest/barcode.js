import isEmail from "validator/lib/isEmail";
import { failResult, restResult, sessionRest } from "../utils/rest";
import { isCallbackKey, isDomain } from "../utils/utils";
import { isHexColor } from "validator";


function barcodeRest() {
    return sessionRest(`api/barcode`)
};

export const createStatus = [
    {
        key: 2,
        val: '公开'
    },
    {
        key: 1,
        val: '私有'
    }
];
export const createBarcodeType = [
    {
        key: "qrcode",
        val: '二维码'
    },
    {
        key: "codabar",
        val: '条形码'
    },
    {
        key: "aztec",
        val: 'Aztec'
    },
    {
        key: "code93",
        val: 'Code 93'
    },
    {
        key: "code128",
        val: 'Code 128'
    },
    {
        key: "datamatrix",
        val: 'Data Matrix'
    },
    {
        key: "ean8",
        val: 'Ean8'
    },
    {
        key: "ean13",
        val: 'Ean13'
    },
    {
        key: "itf",
        val: 'ITF'
    },
    {
        key: "maxicode",
        val: 'Maxi Code'
    },
    {
        key: "pdf417",
        val: 'PDF 417'
    },
    {
        key: "mqr",
        val: 'Micro QR Code'
    },
    {
        key: "rmqr",
        val: 'Rectangular Micro QR Code'
    },
    {
        key: "rss14",
        val: 'GS1 Databar Coupon'
    },
    {
        key: "rss_expanded",
        val: 'Expanded Rss'
    },
    {
        key: "telepen",
        val: 'Telepen'
    },
    {
        key: "upca",
        val: 'UPC A'
    },
    {
        key: "upce",
        val: 'UPC E'
    },
    {
        key: "upc_ean_extension",
        val: 'UPC Ean Extension'
    },
    {
        key: "dxfilmedge",
        val: 'DXFilmEdge'
    },
    {
        key: "DXFilmEdge",
        val: 'DX Film Edge'
    }
];
export const createImageFormat = [
    {
        key: "jpg",
        val: 'Jpg;Jpeg'
    },
    {
        key: "png",
        val: 'Png'
    },
    {
        key: "gif",
        val: 'Gif'
    },
    {
        key: "webp",
        val: 'WebP'
    },
    {
        key: "bmp",
        val: 'BMP'
    }
];



export async function barcodeCreateAdd(param, config) {
    const { app_id, barcode_type, status, image_format,
        image_width, image_height, margin,
        image_color, image_background } = param;
    var errors = {};
    if (app_id <= 0) {
        errors.name = "应用名未选择";
    }
    param.app_id = parseInt(app_id)
    if (!createStatus.find((t) => t.key == status)) {
        errors.status = "类型未选择";
    } else {
        param.status = parseInt(status)
    }
    if (!createBarcodeType.find((t) => t.key == barcode_type)) {
        errors.barcode_type = "格式异常";
    }
    if (!createImageFormat.find((t) => t.key == image_format)) {
        errors.image_format = "格式异常";
    }
    if (image_width <= 0 || image_width > 10000) {
        errors.image_width = "宽度异常";
    } else {
        param.image_width = parseInt(image_width)
    }
    if (image_height <= 0 || image_height > 10000) {
        errors.image_height = "高度异常";
    } else {
        param.image_height = parseInt(image_height)
    }
    if (margin < 0 || margin > image_width / 2 || margin > image_height / 2) {
        errors.margin = "margin异常";
    } else {
        param.margin = parseInt(margin)
    }
    if (!isHexColor(image_color)) {
        errors.barcode_type = "颜色异常";
    }
    if (!isHexColor(image_background)) {
        errors.barcode_type = "背景颜色异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await barcodeRest().post("/create_config_add", param, config);
    return restResult(response)
}

export async function barcodeCreateEdit(param, config) {
    const { id, barcode_type, status, image_format,
        image_width, image_height, margin,
        image_color, image_background } = param;
    var errors = {};
    if (id <= 0) {
        errors.name = "未选择";
    }
    param.id = parseInt(id)
    if (!createStatus.find((t) => t.key == status)) {
        errors.status = "类型未选择";
    }
    if (!createBarcodeType.find((t) => t.key == barcode_type)) {
        errors.barcode_type = "格式异常";
    }
    if (!createImageFormat.find((t) => t.key == image_format)) {
        errors.image_format = "格式异常";
    }
    if (image_width <= 0 || image_width > 10000) {
        errors.image_width = "宽度异常";
    } else {
        param.image_width = parseInt(image_width)
    }
    if (image_height <= 0 || image_height > 10000) {
        errors.image_height = "高度异常";
    } else {
        param.image_height = parseInt(image_height)
    }
    if (margin < 0 || margin > image_width / 2 || margin > image_height / 2) {
        errors.margin = "margin异常";
    } else {
        param.margin = parseInt(margin)
    }
    if (!isHexColor(image_color)) {
        errors.barcode_type = "颜色异常";
    }
    if (!isHexColor(image_background)) {
        errors.barcode_type = "背景颜色异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await barcodeRest().post("/create_config_edit", param, config);
    return restResult(response)
}
export async function barcodeCreateList(param, config) {
    const { id, app_id, barcode_type, page, page_size } = param;
    let data = {
        count_num: true,
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    if (id >= 0) {
        data.id = parseInt(id);
    }
    if (app_id >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (typeof barcode_type == "string" && barcode_type.length > 0) {
        data.barcode_type = barcode_type;
    }
    let response = await barcodeRest().post("/create_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function barcodeCreateDel(param, config) {
    const { id } = param;
    let data = {};
    if (id > 0) {
        data.id = parseInt(id);
    }
    let response = await barcodeRest().post("/create_config_delete", data, config);
    return restResult(response)
}


export async function barcodeParseList(param, config) {
    const { id, app_id, barcode_type, page, page_size } = param;
    let data = {
        count_num: true,
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    if (id >= 0) {
        data.id = parseInt(id);
    }
    if (app_id >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (typeof barcode_type == "string" && barcode_type.length > 0) {
        data.barcode_type = barcode_type;
    }
    let response = await barcodeRest().post("/parse_record_list", data, config);
    return restResult(response, ['not_found'])
}

export async function barcodeParseDel(param, config) {
    const { id } = param;
    let data = {};
    if (id > 0) {
        data.id = parseInt(id);
    }
    let response = await barcodeRest().post("/parse_record_delete", data, config);
    return restResult(response)
}

