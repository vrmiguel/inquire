# Inquire your files on the terminal

## Example usage

```
❯ inquire $(which gcc)
[/usr/bin/gcc]
· ELF 64-bit LSB executable
· x86-64
· version 1 (SYSV)
· dynamically linked
· interpreter /lib64/ld-linux-x86-64.so.2
· BuildID[sha1]=d55fe5f225672a0fb6061c61ab3865a4c8094d4b
· for GNU/Linux 4.4.0
· stripped

[dependencies]
· libm.so.6
· libc.so.6
· ld-linux-x86-64.so.2

[file info]
size:                   1.25 MB                     
permissions:            -rwxr-xr-x                      0755
owner:                  root                            0
owner's group:          root                            0
last modified:          Tuesday Oct/12/2021 18:29:29
last accessed:          Tuesday Oct/12/2021 18:29:29
```

## To-do list

- [x] Use `libmagic` to get information on the file 
- [x] Have a MIME type sniffer fallback that does not rely on `libmagic`
- [x] Show the size of the file
- [x] Show the file permissions
- [x] Show the username of the file's owner
- [x] Show the user group of the file's owner
- [x] If the file is an executable, try to show its dynamic dependencies, if any exist
- [x] Show last modified date
- [x] Show last accessed date
- [ ] Add tests to `wizardry`, our `libmagic` bindings crate.
- [ ] Allow building `libmagic` statically (or at least make ir work properly when built with MUSL)
- [ ] Generally improve the output: better formatting, colors, etc.
- [ ] Command-line options and flags?
