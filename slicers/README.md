# Slicers

This directory contains crates intended to provide a means of interfacing with different 3D printing slicer backends.

## Supported Backends
Currently no slicer backend is supported however the following are planned:
1. cura through cura engine (using command line interface)
2. prusa (slic3r) through embedding a C wrapper around the C++ library.
3. GladiusSlicer (a slicer written in pure rust around a command interface structure that makes it much easier to embed)