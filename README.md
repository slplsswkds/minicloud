### Update:
* Version 0.9.0 and above (commit 37eefdc) does not contain the security issue of accessing files (as was the case in version 0.1.0 with commit 585be84) that were not distributed. This version may still have bugs, but it is ready to use for now.

### Usage:
* default usage: _minicloud ~/path/to/the/file/or/directory_
* specify another port: _minicloud -p 65432 ~/your/path_
* help: _minicloud --help_

### Roadmap
- [x] Scan files
- [x] Scan directories
- [ ] Scan symbolic links
- [x] Generate an HTML page in the form of a tree
- [x] Hashing items in the hyperlinks
- [ ] Chunked downloading
- [x] Linux support
- [x] Windows support
- [ ] Downloading the entire directory
