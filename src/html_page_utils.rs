use crate::fs_object::{FsObject, FsObjects};
use std::collections::HashMap;
use std::sync::Arc;

/// Returns html unordered list from [`FsObjects`] recursively
pub fn unordered_list(files: &FsObjects, hash_map: &mut HashMap<u64, Arc<FsObject>>) -> String {
    let list_of_items = list_of_items(files, hash_map);
    format!("<ul>\n{}</ul>\n", list_of_items)
}

/// Returns the html code for the list from &[`FsObjects`]
fn list_of_items(items: &FsObjects, hash_map: &mut HashMap<u64, Arc<FsObject>>) -> String {
    let mut list = String::new();

    for item in items.iter() {
        if item.is_dir() {
            list += &list_item(&item, hash_map);
            if let Some(content) = &item.content {
                let unordered_list = unordered_list(&content, hash_map);
                list += &unordered_list;
            }
        } else {
            list += &list_item(item, hash_map);
        }
    }

    list
}

/// Returns the html code for one list item and adds the key-value pair for that item to the HashMap
fn list_item(item: &Arc<FsObject>, hash_map: &mut HashMap<u64, Arc<FsObject>>) -> String {
    if item.is_dir() {
        format!("<li>📁 {}</li>\n", item.name())
    } else if item.is_symlink() {
        format!("<li>🔗 {}</li>\n", item.name())
    } else {
        let hash_key = item.get_hash();
        hash_map.insert(hash_key, Arc::clone(item));

        format!(
            "<li>🗋 {}, {} {}</li>\n",
            href(item.name().as_ref(), url_download_item(hash_key).as_ref()),
            item.size(),
            href("[view]", url_preview_item(hash_key).as_ref()),
        )
    }
}

/// Create HTML href from Text and URL
fn href(text: &str, url: &str) -> String {
    format!("<a href=\"{}\">{}</a>", url, text)
}

/// Create URL to download item(file/folder) by its hash
/// URL format: "/dl?id={}", where {} is Hash of FSObject
fn url_download_item(hash: u64) -> String {
    format!("/dl?id={}", hash)
}

/// Create URL to preview item(file/folder) by its hash
/// URL format: "/pw?id={}", where {} is Hash of FSObject
fn url_preview_item(hash: u64) -> String {
    format!("/pw?id={}", hash)
}
