use crate::fs_object::FSObject;
use axum::{extract::State, http::header, response::Html};
use std::path::PathBuf;

// Returns a generated line of HTML code that contains a list of files in the form of a tree
fn unordered_list_tree(fsobjects: &Vec<FSObject>) -> String {
    let mut list = String::new();
    fsobjects.iter().for_each(|fsobject| {
        list += "<li>";
        if fsobject.is_dir() {
            list += &format!(
                "<span class=\"caret\">{}</span>\n<ul class=\"nested\">\n",
                fsobject.name()
            );
            let embeded_list = match &fsobject.content {
                Some(l) => unordered_list_tree(l),
                None => String::new(),
            };
            list += &embeded_list;
            list += "</ul>\n";
        } else {
            list += fsobject.name()
        }
        list += "</li>\n"
    });
    list
}

fn meta() -> String {
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
    let head = format!("<head>\n{}\n{}\n</head>", meta(), style());
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
        unordered_list_tree(fsobjects),
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
