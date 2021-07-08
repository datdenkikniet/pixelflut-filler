# Pixelflut-filler

A bulk-write filler for a pixelflut instance.

Can sample some random data from `/dev/urandom` to use as filling color.

This program simply loads all commands into memory, and sends
them all in one go to minimize syscalls and overhead.

# Usage
```
pixelflut-filler [remote IPv4] [color]
    
    [remote IPv4]  The IPv4 address of the remote (Optional, default 127.0.0.1)

    [color]        The color to use for the fill, either in 6 or 8 character
                   hex. (Optional, default samples from /dev/urandom)
```