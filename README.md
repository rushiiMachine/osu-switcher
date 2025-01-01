# osu-switcher ![views](https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2FDiamondMiner88%2Fosu-server-switcher&count_bg=%2379C83D&title_bg=%23555555&icon=github.svg&icon_color=%23E7E7E7&title=views&edge_flat=true)

- Switch between accounts for different servers seamlessly via desktop shortcuts
- Shortcuts will quickly relaunch osu!

## Usage

1. Download the [latest release](https://github.com/DiamondMiner88/osu-switcher/releases/latest)
2. Double click run to set up shortcuts for multiple servers
3. Use the shortcuts on your desktop to launch osu!

## Additional Info

- I don't sign my releases, Windows SmartScreen may appear on first launch.
- Due to an osu!stable bug you need to sign-in **twice** across different osu! launches before your credentials for a
  specific server can be saved.
- Shortcuts will have to be recreated if you've moved the location of the following:
    - `osu!` install directory
    - `osu-switcher.exe`

## How does it work?

osu! stores auth details in the `<osu-dir>/osu!.<username>.cfg` config file, under the following keys:

- `Username` -> Public username
- `Password` -> A login session specific key
- `CredentialEndpoint` -> The auth server these credentials are meant for

In short, the created shortcuts launch this switcher program, which backs up the credentials for the currently logged in
server, and restores saved credentials for the target server from the custom `<osu-dir>/osu!switcher.ini` config file.

Additionally, in the `<osu-dir>/osu!.db` binary database, there is an additional field that stores the last
login username. If this does not match the `Username` from the config file, then you are automatically logged out.
This is changed when switching logins as well.

## Disclaimer

**This is NOT a tool for multi-accounting.**\
I often play on private servers often find it annoying to keep re-entering my login details.
This tool allows you to switch accounts *between* servers, not on the same server.
This tool does not spoof any device fingerprinting in order to bypass multi-account detection.
