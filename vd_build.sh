#!/bin/bash

/tmp/pvk/debug/vk_stdout /home/chris/gl/Vulkan-Docs/xml/vk.xml >/tmp/tst.rs
rustfmt /tmp/tst.rs 2>/tmp/out.txt
rustc /tmp/tst.rs 2>>/tmp/out.txt