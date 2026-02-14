# AUR Browser

A modern GTK4 AUR package browser for any arch based distro (No artix just because you dont use systemd doesnt mean you get used)

## Features

-  Search AUR packages with real-time results
-  Sort by popularity, votes, alphabetical, or last modified
-  View detailed package information (dependencies, conflicts, maintainer)
-  Bookmark favorite packages
- ✓ Shows which packages are already installed
- Opens terminal for easy installation (most terminals accepted)

## Requirements

- Arch Linux or derivative (CachyOS, Manjaro, etc.)
- `yay` AUR helper
- GTK4 and libadwaita
- Internet 

## Installation

### From source:
```bash
# Install dependencies
sudo pacman -S gtk4 libadwaita rust

# Clone and build
git clone https://github.com/viewerofall/aurbrowser.git
cd aurbrowser
cargo build --release


```

## Usage

1. Search for packages using the search bar (press Enter)
2. Click "Details" to view package information
3. Click "Install" to open a terminal and install with yay
4. Use the sort dropdown to organize results
5. Click the star to bookmark packages
6. Click "★ Bookmarks" to view your saved packages

## TO DO
[] Add nixos store and flakes
[] Internal terminal 
[] Full preview, little interaction needed
[] Open into aur page (leads to the aur page of desired item)
[] Detects aur helper and has interaction to use desired one

## License

MIT

## Contributing

Please add things I dont have much time left before he takes me
