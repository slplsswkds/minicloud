//! This module created to generate HTML page with list of files in tree FSObject
use crate::fs_object::FSObject;

/// Returns a generated line of HTML code that contains a list of files in the form of a tree
fn unordered_list_tree(fsobject: &Vec<FSObject>) -> String {
    let mut list = String::new();
    fsobject.iter().for_each(|fsobject| {
        list += "<li>";
        if fsobject.is_dir() {
            list += &format!(
                "<span class=\"caret\">{}</span>\n<ul class=\"nested\">\n",
                fsobject.name()
            );
            let embedded_list = match &fsobject.content {
                Some(l) => unordered_list_tree(l),
                None => String::new(),
            };
            list += &embedded_list;
            list += "</ul>\n";
        } else {
            list += fsobject.name()
        }
        list += "</li>\n"
    });
    list
}

fn configure_viewport() -> String {
    String::from("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">")
}

fn style() -> String {
    String::from(
        "<style>
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

fn head() -> String {
    let head = format!("<head>\n{}\n{}\n</head>", configure_viewport(), style());
    head
}

fn script() -> String {
    String::from(
        "<script>
    var toggler = document.getElementsByClassName(\"caret\");
    var i;

    for (i = 0; i < toggler.length; i++) {
      toggler[i].addEventListener(\"click\", function () {
        this.parentElement.querySelector(\".nested\").classList.toggle(\"active\");
        this.classList.toggle(\"caret-down\");
      });
    }
</script>\n",
    )
}

pub fn body(fsobjects: &Vec<FSObject>) -> String {
    format!(
        "\n<body>\n<ul>{}</ul>\n{}\n</body>\n",
        unordered_list(fsobjects),
        script()
    )
}

pub fn html_page(fsobjects: &Vec<FSObject>) -> String {
    format!(
        "<!DOCTYPE html>\n<html>\n{}\n{}\n</html>\n",
        head(),
        body(fsobjects)
    )
}

/// Returns html unordered list from [`Vec<FSOBject>`] recursively
pub fn unordered_list(files: &Vec<FSObject>) -> String {
    let list_of_items = list_of_items(files);
    format!("<ul>\n{}</ul>\n", list_of_items)
}

/// Returns the html code for the list from &[`Vec<FSObject>`]
fn list_of_items(items: &Vec<FSObject>) -> String {
    let mut list = String::new();
    for item in items.iter() {
        if item.is_dir() {
            list += &list_item(&item);
            match &item.content {
                Some(content) => list += &unordered_list(&content), // += list_of_items for non-tree
                None => {}
            };
        } else {
            list += &list_item(item);
        }
    }
    list
}

/// Returns the html code for one list item
fn list_item(item: &FSObject) -> String {
    let list_item = item.name();
    format!("<li>{}</li>\n", list_item)
}