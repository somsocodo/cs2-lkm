# CS2-LKM

An open source cheat for Counter-Strike 2 on Linux.

### Features

- Aimbot
- Triggerbot
- ESP (nametags/health, skeleton, hitbox)
- Grenade helper

Please see the [wiki](https://github.com/somsocodo/cs2-lkm/wiki) for more in-depth information on features.

## Warning & Disclaimer

By using this software, you acknowledge the following:

- Use of this program may result in penalties, including account suspension or permanent bans.
- I, the developer, am not responsible for any actions taken using this software, nor for any resulting penalties. The responsibility of using this software lies entirely with the user.

## Dependencies

- ```libevdev-devel``` required for rdev mouse grabbing.
- [```mem-kernel-module```](https://github.com/somsocodo/mem-kernel-module) required for reading process memory.
- [```cs2-dumper```](https://github.com/a2x/cs2-dumper) required updating offsets.

## Setup

- clone cs2-dumper to project root ```git clone git@github.com:a2x/cs2-dumper.git -b linux```
- build cs2-dumper ```cd cs2-dumper && cargo build```
- run update.sh in project root ``sudo bash update.sh``

## Usage

- Insure mem-kernel-module is loaded. [README](https://github.com/somsocodo/mem-kernel-module/blob/master/README.md)
- Run as superuser ```sudo cargo run --release```

## Issues/Suggestions

This program is a work in progress, please feel free to open an issue or pull request if you have any suggestions.

Please note there is no plans for Windows support.

## References

- https://github.com/ekknod/EC
- https://unknowncheats.me
