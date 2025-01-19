### Update:
* Version 0.9.0 and above (commit 37eefdc) does not contain the security issue of accessing files (as was the case in version 0.1.0 with commit 585be84) that were not distributed. This version may still have bugs, but it is ready to use for now.
* With the addition of the receiver mode in version 0.12.0, the program lost the graphical user interface for selecting the path to start the server (as it was up to and including version 0.11.3).

### Building:
git clone https://github.com/slplsswkds/minicloud.git && \
cd minicloud && \
cargo build --release

### Usage:
**help:** _minicloud --help_

###### Receive files mode:
* minicloud --receive --received-files-path=/tmp/minicloud ~/path/to/saved/files/

###### Transmit files mode:
* default usage: _minicloud ~/path/to/the/file/or/directory_

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
- [x] Receiving files from clients