# idasen-tui

`idasen-tui` is a Rust-based TUI (Text-based User Interface) tool designed to manage the Idasen sitting-standing desk by Ikea. With this tool, you can conveniently control your desk, including listing available Bluetooth devices, connecting to your Idasen desk, managing favorite positions, and viewing real-time height adjustments.

![Example screenshot](https://github.com/AzraelSec/idasen-tui/blob/main/assets/screen.png?raw=true)

## Features

> [!WARNING]
> `idasen-tui` is my first venture into Rust programming, and as such, it may not adhere to the best practices or standards of Rust development.

- **Bluetooth Device Management**: Easily list available Bluetooth devices and connect to your Idasen desk.
- **Favorite Positions Management**: Store your favorite desk positions (expressed in cm) and favorite device (via its MAC address) using a simple configuration file (`~/.idasen-tui.json`).
- **Real-time Height Display**: Monitor the height adjustments of your Idasen desk in real-time.

## Installation

1. Ensure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).
2. Clone this repository:
   ```
   git clone https://github.com/AzraelSec/idasen-tui.git
   ```
3. Navigate into the project directory:
   ```
   cd idasen-tui
   ```
4. Build the project:
   ```
   cargo build --release
   ```
5. Run the executable:
   ```
   ./target/release/idasen-tui
   ```

## Usage

Upon running `idasen-tui`, you will be presented with a text-based interface providing various options:

- Use tab/reverse tab to navigate through the sections.
- Follow on-screen instructions to connect to your Idasen desk, manage favorite positions, and view real-time height adjustments.

## Configuration

`idasen-tui` utilizes a JSON configuration file located at `~/.idasen-tui.json`. Below is an example of the configuration structure:

```json
{
  "predefined_mac": "XX:XX:XX:XX:XX:XX",
  "saved_positions":[
    {
      "name": "sitting",
      "height": 6691
    },
    {
      "name": "standing",
      "height": 10575
    }
  ]
}
```

- **predefined_mac**: MAC address of the default Bluetooth device for your Idasen desk.
- **saved_positions**: Array containing favorite desk positions, where each position object consists of a name and corresponding height (expressed in mm).

> [!NOTE]
> On Windows, configuration file should be positioned in `C:\Users\<username>`

## Contributing

Contributions are welcome! If you have any ideas for new features, improvements, or bug fixes, feel free to open an issue or submit a pull request.

## Credits

This project was made possible thanks to the following libraries:

- [**idasen**](https://github.com/aklajnert/idasen): A Rust library that provides abstractions for controlling the Idasen sitting-standing desk by Ikea.
- [**Ratatui**](https://ratatui.rs/): One of the most complete and feature-proof TUI library.
- [**tui-big-text**](https://github.com/joshka/tui-big-text): A crate a rust crate that renders large pixel text as a Ratatui widget using the glyphs from the font8x8 crate.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
