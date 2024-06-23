use std::collections::HashMap;
use std::path::Path;
use lazy_static::lazy_static;

pub const CONTENT_TYPE_HEADER: &str = "Content-Type";
pub const DEFAULT_CONTENT_TYPE: &str = "text/plain";
pub const DEFAULT_STATUS_CODE : &str = "OK";

lazy_static! {
    pub static ref CONTENT_TYPE_MAP: HashMap<&'static str, &'static str> = {
        let mut result = HashMap::new();

        // Common text types
        result.insert("txt", DEFAULT_CONTENT_TYPE);
        result.insert("html", "text/html");
        result.insert("css", "text/css");
        result.insert("xml", "text/xml");
        result.insert("js", "application/javascript");

        // Common image types
        result.insert("gif", "image/gif");
        result.insert("jpeg", "image/jpeg");
        result.insert("jpg", "image/jpeg");
        result.insert("png", "image/png");
        result.insert("svg", "image/svg+xml");
        result.insert("ico", "image/x-icon");

        // Common audio and video types
        result.insert("mp3", "audio/mpeg");
        result.insert("ogg", "audio/ogg");
        result.insert("mp4", "video/mp4");
        result.insert("mpeg", "video/mpeg");
        result.insert("mov", "video/quicktime");
        result.insert("webm", "video/webm");

        // Common application types
        result.insert("json", "application/json");
        result.insert("pdf", "application/pdf");
        result.insert("zip", "application/zip");
        result.insert("doc", "application/msword");
        result.insert("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        result.insert("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        result.insert("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation");

        result
    };

    pub static ref STATUS_CODES_MAP: HashMap<usize, &'static str> = {
        let mut result = HashMap::new();

        result.insert(200, DEFAULT_STATUS_CODE);
        result.insert(201, "Created");
        result.insert(202, "Accepted");
        result.insert(204, "No Content");
        result.insert(400, "Bad Request");
        result.insert(401, "Unauthorized");
        result.insert(403, "Forbidden");
        result.insert(404, "Not Found");
        result.insert(405, "Method Not Allowed");
        result.insert(500, "Internal Server Error");
        result.insert(502, "Bad Gateway");
        result.insert(503, "Service Unavailable");

        result
    };
}