#!/usr/bin/env python3
import sys
import os

sys.path.insert(0, "@out@/lib/python3.13/site-packages")

from controller import main

if __name__ == "__main__":
    config_path = (
        sys.argv[1]
        if len(sys.argv) > 1
        else "@out@/share/hid-digital-lcd-controller/config.json"
    )
    main(config_path)
