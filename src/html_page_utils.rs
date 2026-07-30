use crate::fs_object::{FsObject, FsObjects};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

pub fn unordered_list(files: &FsObjects, hash_map: &mut HashMap<u64, Arc<FsObject>>) -> Box<str> {
    let mut html_buf = String::with_capacity(files.len() * 100 + 32);

    render_unordered_list(files, hash_map, &mut html_buf);

    html_buf.into_boxed_str()
}

fn render_unordered_list(
    files: &FsObjects,
    hash_map: &mut HashMap<u64, Arc<FsObject>>,
    buf: &mut String,
) {
    buf.push_str("<ul>\n");
    render_list_items(files, hash_map, buf);
    buf.push_str("</ul>\n");
}

fn render_list_items(
    items: &FsObjects,
    hash_map: &mut HashMap<u64, Arc<FsObject>>,
    buf: &mut String,
) {
    for item in items {
        if item.is_dir() {
            let _ = writeln!(buf, "<li>📁 {}</li>", item.name());
            if let Some(content) = &item.content {
                render_unordered_list(content, hash_map, buf);
            }
        } else if item.is_symlink() {
            let _ = writeln!(buf, "<li>🔗 {}</li>", item.name());
        } else {
            let hash = item.get_hash();
            hash_map.insert(hash, Arc::clone(item));

            let _ = writeln!(
                buf,
                r#"<li>🗋 <a href="/dl?id={hash}">{}</a>, {} <a href="/pw?id={hash}">[view]</a></li>"#,
                item.name(),
                item.size_string()
            );
        }
    }
}