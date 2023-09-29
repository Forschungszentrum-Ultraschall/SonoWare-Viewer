# SonoWare Viewer
This repository contains the source code of a lightweight viewer for ultrasonic data.

For further help using the viewer, read the 
integrated help page.

## Installation
### Pre-build executables
1. Download the latest [Release](https://github.com/Forschungszentrum-Ultraschall/US-Viewer/releases/tag/0.4.3) and unzip it.
2. Execute the `sonoware-viewer` binary. **WARNING**: As the binaries
aren't signed you'll get an warning message on Windows devices.

### Sourcecode
1. Install the latest [Rust release](https://www.rust-lang.org/).
2. Clone this repository.
3. Run the program by executing the command
   ```shell
   cargo run --release
   ```