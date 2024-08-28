//! This module created to generate HTML page with list of files in tree FSObject

use std::collections::HashMap;
use crate::fs_object::FSObject;

pub fn html_page(fsobjects: &Vec<FSObject>) -> String {
    format!(
        "<!DOCTYPE html>
<html>
{}
{}
</html>",
        head(),
        body(fsobjects)
    )
}

//------------------------------ Head elements -----------------------------------------------------
/// Configure <title>, <style>, <meta>, <link>, <script>, and <base>. Not finished
#[inline]
fn head() -> String {
    format!(
        "<head>
            {}
            {}
            {}
            {}
            {}
            {}
        \r</head>\n",
        title(),
        style(),
        meta(),
        link(),
        script(),
        base()
    )
}

#[inline]
fn title() -> String {
    format!("\n<title> Minicloud v{} </title>\n", env!("CARGO_PKG_VERSION"))
}

#[inline]
fn style() -> String {
    String::from("\n<style>
    ul,
    #myUL {
      list-style-type: none;
    }

    #myUL {
      margin: 0;
      padding: 0;
    }

    .caret::before {
      content: \"\\25B6\";
      color: black;
      display: inline-block;
      margin-right: 5px;
    }

    .caret-down::before {
      -ms-transform: rotate(90deg);
      /* IE 9 */
      -webkit-transform: rotate(90deg);
      /* Safari */
      transform: rotate(90deg);
    }

    .nested {
      display: none;
    }

    .active {
      display: block;
    }
</style>",
    )
}

#[inline]
fn meta() -> String {
    "\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n".to_string()
}

#[inline]
fn link() -> String {
    "\n".to_string()
}

#[inline]
fn script() -> String {
    "\n<script>
    var toggler = document.getElementsByClassName(\"caret\");
    var i;
    for (i = 0; i < toggler.length; i++) {
        toggler[i].addEventListener(\"click\", function () {
            this.parentElement.querySelector(\".nested\").classList.toggle(\"active\");
            this.classList.toggle(\"caret-down\");
        });
    }
</script>\n".to_string()
}

#[inline]
fn base() -> String {
    //<base href="https://www.w3schools.com/" target="_blank">
    "".to_string()
}

//--------------------------------------------------------------------------------------------------
pub fn body(fsobjects: &Vec<FSObject>) -> String {
    format!("<body>
    \r{}
</body>", unordered_list(fsobjects)
    )
}

/// Returns html unordered list from [`Vec<FSObject>`] recursively
pub fn unordered_list(files: &Vec<FSObject>) -> String {
    let list_of_items = list_of_items(files);
    format!("<ul>\n{}</ul>\n", list_of_items)
}

/// Returns the html code for the list from &[`Vec<FSObject>`]
fn list_of_items(items: &Vec<FSObject>) -> String {
    let mut hashes = HashMap::new();

    let mut list = String::new();
    for item in items.iter() {
        if item.is_dir() {
            list += &list_item(&item, &mut hashes);
            match &item.content {
                Some(content) => list += &unordered_list(&content), // += list_of_items for non-tree
                None => {}
            };
        } else {
            list += &list_item(item, &mut hashes);
        }
    }
    list
}

/// Returns the html code for one list item and adds the key-value pair for that item to the HashMap
fn list_item<'a>(item: &'a FSObject, hash_map: &mut HashMap<u64, &'a FSObject>) -> String {
    let list_item: String;
    if item.is_dir() {
        list_item = format!("<li>📁 {}</li>\n", item.name())
    } else if item.is_symlink() {
        list_item = format!("<li>🔗 {}</li>\n", item.name())
    } else {
        let hash_key = item.get_hash();
        hash_map.insert(hash_key, item);

        list_item = format!(
            "<li>🗋 {}, {}</li>\n",
            href(item.name().as_ref(), url_download_item(hash_key).as_ref()),
            item.size()
        )
    }
    list_item
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