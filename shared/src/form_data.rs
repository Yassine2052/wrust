use serde_json::Value;

#[derive(Debug)]
pub struct File {
    name: String,
    field_name: String,
    original_name: String,
    mime_type: String,
    destination: String,
    path: String,
    encoding: String,
    size: usize
}

#[derive(Debug)]
pub struct  FormData {
    data: Value,
    files: Vec<File>
}