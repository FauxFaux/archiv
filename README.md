## archiv

`archiv` is a library and CLI for working with `.archiv` files.

`.archiv` files are a simplistic "file format" that keeps getting
reinvented, by me.

This file format is:

A header, then, repeatedly:
 * `length`, 8 bytes, little-endian: the length of the following data
 * `data`, the data

You've probably heard of this before.

On top of this we can layer things like:

 * [x] stream compression
 * [x] item compression
 * [x] item compression with a shared dictionary
 * [ ] item compression with an embedded dictionary
 * [ ] parallel processing of item compressed files
 * [ ] indexes
 * [ ] docs and shared terminology


### Contributing

Github.


### License

MIT / Apache-2
