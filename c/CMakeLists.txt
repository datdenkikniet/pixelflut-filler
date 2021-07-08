cmake_minimum_required(VERSION 3.16)
project(pixelflut_filler C)

set(CMAKE_C_STANDARD 99)

add_executable(pixelflut_filler main.c color.h color.c window.h window.c)