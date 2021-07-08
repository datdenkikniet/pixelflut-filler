# Pixelflut-filler

A bulk-write filler for a pixelflut instance running on the localhost.

Samples some random data from `/dev/urandom` to use as filling color.

This program simply loads all commands into memory, and sends
them all in one go to minimize syscalls and overhead.