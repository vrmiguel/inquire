# Inquire your files on the terminal

## Example usage

```
❯ inquire $(which rg)
[/usr/bin/rg]
· ELF 64-bit LSB pie executable
· x86-64
· version 1 (SYSV)
· dynamically linked
· interpreter /lib64/ld-linux-x86-64.so.2
· BuildID[sha1]=b03ef3339bd12b330f2e3d5973613dedd8581cab
· for GNU/Linux 4.4.0
· stripped
size: 4.55 MB
permissions: -rwxr-xr-x
owner's username: root
owner's group: root
```

## To-do list

- [x] Use `libmagic` to get information on the file 
- [x] Have a MIME type sniffer fallback that does not rely on `libmagic`
- [x] Show the size of the file
- [x] Show the file permissions
- [x] Show the username of the file's owner
- [x] Show the user group of the file's owner
- [ ] Show last modified date
- [ ] Show last accessed date