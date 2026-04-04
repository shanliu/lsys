//! lsys-sdk - 内部服务间调用 SDK
//!
//! 提供 HTTP 客户端用于服务间通信，使用签名认证。

mod client;
mod result;
mod types;
mod utils;

// 业务模块
mod impls;
pub use client::{ServiceClient, ServiceRequest};
pub use result::{ApiError, ApiErrorDetail, HttpRejectedError, ParseError, ServiceError};
pub use types::{
    ForwardedRequest, ReqInfo, ACCEPT_LANGUAGE_HEADER, DEVICE_ID_HEADER, FORWARDED_FOR_HEADER,
    REQUEST_ID_HEADER,
};

// 业务类型导出
pub use impls::app::{AppFeatureResponse, AppSecretResponse};
pub use impls::auth::{AuthVerifyParam, AuthVerifyResponse};
pub use impls::file::{
    CursorResp, FileChunkParam, FileFromLocalResponse, FileFromUrlResponse, FileInfoItem,
    FileInfoResponse, FileListItem, FileListResponse, FileMappingResponse, FileTagItem,
    FileUploadByMd5Response, FileUploadCreateResponse, FileUploadRetokenResponse, FileUrlsResponse,
    TotalResp,
};
pub use impls::rbac::{
    AccessCheckParam, RbacCheckItem, RbacCheckParam, RbacCheckRequest, RbacCheckResponse,
    RbacCheckStatus, ResCheckParam, ResReqAuthParam, RoleCheckParam,
};
