# LofiTurtle 🎵

A beautiful terminal-based music player built with Rust, featuring a modern TUI interface powered by Ratatui.

## Features

- **🎵 Audio Playback**: Supports MP3, FLAC, AAC, M4A, OGG, and WAV formats with rodio 0.21.1
- **🔍 Smart Search**: Real-time search by song title, artist, and album
- **📚 Music Library**: Automatic scanning and SQLite-based music library management
- **🎨 Beautiful TUI**: Clean, responsive 3-panel interface with progress bars and controls
- **🎵 Playlist Management**: Create, edit, delete, and manage custom playlists
- **🎨 Album Art Display**: ASCII art rendering of album artwork with toggleable display
- **🔀 Shuffle Mode**: Random song playback with fair randomization algorithm
- **🔁 Repeat Modes**: Single song and playlist repeat options
- **⚡ Gapless Playback**: Seamless transitions between songs (enhanced audio player)
- **🎚️ Volume Control**: Adjustable audio volume with visual indicators and persistence
- **⚡ Performance Optimized**: String caching, album art caching, and memory optimizations
- **🖥️ Cross-Platform**: Works on macOS, Linux, and Windows

## Screenshots

### 3-Panel Interface Layout
```
┌Search Library──────────────────────────────────────────────────────────────┐
│Search songs...                                                             │
└────────────────────────────────────────────────────────────────────────────┘
┌Playlists──────┬─Songs (42/42)──────────────────┬─Album Art───────────────────┐
│   📚 All Songs │   Bohemian Rhapsody - Queen    │                             │
│>> My Playlist  │>> Stairway to Heaven - Led Zep │     ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫       │
│   Rock Hits    │   Hotel California - Eagles   │   ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫     │
│   Favorites    │   Sweet Child O' Mine - GNR    │ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪   │
│                │                                │   ♫ ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫     │
│                │                                │     ♪ ♫ ♪ ♫ ♪ ♫ ♪ ♫       │
└────────────────┴────────────────────────────────┴─────────────────────────────┘
┌Controls────────────────────────────────────────────────────────────────────┐
│♪ Stairway to Heaven - Led Zeppelin                                         │
│████████████████████████████████████████████████████████ 03:45 / 08:02     │
│Tab: Switch panels | Enter: Play | Space: Play/Pause | [/]: Volume | q: Quit│
│▶ Playing                                                                   │
│Shuffle: 🔀 OFF  |  Repeat: 🔁 OFF  |  Volume: 🔊 75%                        │
└────────────────────────────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Audio system (ALSA on Linux, CoreAudio on macOS, WASAPI on Windows)

### Building from Source

```bash
git clone https://github.com/uy-td-dev/LoFi-Turtle.git
cd lofiturtle
cargo build --release
```

## Usage

### Basic Usage

```bash
# Run with default music directory
cargo run

# Specify custom music directory
cargo run -- --music-dir /path/to/your/music

# Or after building
./target/release/lofiturtle --music-dir /path/to/your/music

# Show help for all available commands and options
./target/release/lofiturtle --help
```

### Command Line Interface

LofiTurtle offers both interactive TUI mode and command-line operations:

```bash
# Interactive music player (default)
lofiturtle play

# Scan music library and update database
lofiturtle scan

# List all songs in the database
lofiturtle list

# Search for songs
lofiturtle search "artist name" or "song title"

# Manage playlists
lofiturtle playlist create "My Playlist"
lofiturtle playlist list
lofiturtle playlist add "My Playlist" "song1" "song2"
lofiturtle playlist play "My Playlist"

# Enable features
lofiturtle --shuffle --repeat playlist --show-art
```

### Default Music Directories

- **macOS**: `/Users/Shared/Music`
- **Linux**: `/home/music`
- **Windows**: `C:\Users\Public\Music`

### Keyboard Controls

#### Panel Navigation
- `Tab`: Switch between panels (Playlists → Songs → Album Art)
- `hjkl` or `↑↓←→`: Navigate within panels
- `Backspace`: Go back to previous panel/mode

#### Playback Controls
- `Enter`: Play selected song or select playlist
- `Space`: Toggle play/pause
- `s`: Stop playback
- `S`: Toggle shuffle mode
- `R`: Cycle repeat modes (Off → Single → Playlist)

#### Volume Controls
- `[`: Decrease volume by 10%
- `]`: Increase volume by 10%
- Volume displays with icons: 🔇 (mute), 🔈 (low), 🔉 (medium), 🔊 (high)

#### Search & UI
- `/`: Enter search mode
- `c`: Clear search
- `a`: Toggle album art display
- `q`: Quit application

#### Playlist Management
- `n`: Create new playlist (when in Playlists panel)
- `d`: Delete selected playlist
- `+`: Add song to playlist (when in Songs panel)
- `-`: Remove song from playlist

#### Search Mode
- `Type`: Search songs by title, artist, or album
- `Esc`: Exit search mode
- `Enter`: Play selected song and exit search

#### Playlist Input Mode
- `Type`: Enter playlist name
- `Enter`: Create/save playlist
- `Esc`: Cancel operation

## Project Structure

```
src/
├── main.rs              # Application entry point and CLI dispatcher
├── cli.rs               # Command-line argument parsing with clap
├── config.rs            # Configuration management with Builder pattern
├── error.rs             # Custom error types with thiserror
├── art.rs               # Album art extraction and ASCII rendering
├── models/
│   ├── mod.rs          # Module exports
│   ├── song.rs         # Song data structure
│   ├── playlist.rs     # Playlist models and builder
│   └── playback.rs     # Playback state and strategies
├── library/
│   ├── mod.rs          # Module exports
│   ├── database.rs     # SQLite database operations with playlist support
│   └── scanner.rs      # Music directory scanning
├── audio/
│   ├── mod.rs          # Module exports
│   ├── player.rs       # Basic audio playback with Rodio
│   └── gapless_player.rs # Enhanced gapless audio player
├── commands/
│   ├── mod.rs          # Command pattern implementation
│   ├── play.rs         # Play command
│   ├── scan.rs         # Scan command
│   ├── list.rs         # List command
│   ├── search.rs       # Search command
│   └── playlist.rs     # Playlist management commands
├── services/
│   ├── mod.rs          # Service layer
│   └── tui_service.rs  # TUI management service
└── ui/
    ├── mod.rs          # Module exports
    ├── app.rs          # Application state management
    └── widgets.rs      # TUI interface components
```

## Technical Details

### Dependencies

- **ratatui**: Modern TUI framework for beautiful terminal interfaces
- **crossterm**: Cross-platform terminal manipulation
- **rodio**: Pure Rust audio playback library (v0.21.1 compatible)
- **lofty**: Fast metadata extraction for audio files and album art
- **rusqlite**: SQLite database bindings with playlist support
- **tui-textarea**: Text input widget for search functionality
- **clap**: Command-line argument parsing with derive macros
- **thiserror**: Custom error type definitions
- **tokio**: Async runtime for enhanced audio player
- **image**: Image processing for album art rendering

### Database Schema

```sql
CREATE TABLE songs (
    id TEXT PRIMARY KEY,        -- MD5 hash of file path
    path TEXT NOT NULL UNIQUE,  -- Full path to audio file
    title TEXT NOT NULL,        -- Song title
    artist TEXT NOT NULL,       -- Artist name
    album TEXT NOT NULL,        -- Album name
    duration INTEGER NOT NULL   -- Duration in seconds
);

CREATE TABLE playlists (
    id TEXT PRIMARY KEY,        -- UUID for playlist
    name TEXT NOT NULL UNIQUE,  -- Playlist name
    description TEXT,           -- Optional description
    created_at TEXT NOT NULL,   -- ISO 8601 timestamp
    updated_at TEXT NOT NULL    -- ISO 8601 timestamp
);

CREATE TABLE playlist_songs (
    playlist_id TEXT NOT NULL,  -- Foreign key to playlists
    song_id TEXT NOT NULL,      -- Foreign key to songs
    position INTEGER NOT NULL,  -- Order in playlist
    added_at TEXT NOT NULL,     -- ISO 8601 timestamp
    PRIMARY KEY (playlist_id, song_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
);
```

### Architecture

- **Multi-threaded**: Separate threads for UI, audio playback, and file scanning
- **Event-driven**: Responsive UI with efficient event handling
- **Safe concurrency**: Uses channels (mpsc) for thread communication
- **Error handling**: Comprehensive error handling with custom `thiserror` types
- **Design Patterns**: Command, Builder, Strategy, and Facade patterns implemented
- **Clean Code**: Follows Rust best practices and naming conventions
- **Modular Design**: Separated concerns with dedicated modules and services

## Supported Audio Formats

- **MP3**: MPEG Audio Layer III
- **FLAC**: Free Lossless Audio Codec
- **AAC**: Advanced Audio Coding
- **M4A**: MPEG-4 Audio
- **OGG**: Ogg Vorbis
- **WAV**: Waveform Audio File Format

## Performance

- **Fast startup**: Incremental database updates (only new files are processed)
- **Efficient search**: SQLite-powered search with indexing
- **Low memory**: Streaming audio playback without loading entire files
- **Responsive UI**: 60 FPS interface updates

## Troubleshooting

### Audio Issues

**No sound on Linux:**
```bash
# Install ALSA development libraries
sudo apt-get install libasound2-dev

# Or for PulseAudio
sudo apt-get install libpulse-dev
```

**Permission errors:**
- Ensure the music directory is readable
- Check file permissions for audio files

### Database Issues

**Corrupted database:**
```bash
# Remove the database file to rebuild
rm music_library.db
cargo run -- /path/to/music
```

### Performance Issues

**Slow scanning:**
- Large music libraries (>10,000 songs) may take time on first scan
- Subsequent runs are faster due to database caching
- Consider excluding unnecessary subdirectories

## Recent Updates & Achievements

### ✅ Completed Features (v1.0)

- **🎵 Playlist Management**: Full CRUD operations for custom playlists with UI
- **🎨 Album Art Display**: ASCII art rendering with caching and toggleable display
- **🔀 Shuffle Mode**: Random song playback with fair randomization algorithm
- **🔁 Repeat Modes**: Single song and playlist repeat options with UI indicators
- **⚡ Gapless Playback**: Enhanced audio player with seamless transitions
- **🎚️ Volume Control**: Complete volume control with visual icons and persistence
- **⚡ Performance Optimizations**: String caching, album art caching, memory optimizations
- **🎨 3-Panel Interface**: Playlists, Songs, and Album Art panels with Tab navigation
- **🏗️ Architecture Refactor**: Applied clean code principles and design patterns
- **🔧 Rodio 0.21.1 Compatibility**: Updated to latest rodio API
- **📋 Enhanced CLI**: Comprehensive command-line interface with help system
- **🔍 Enhanced Search**: Search by title, artist, and album with real-time filtering

### 🚀 Future Enhancements

- **🎚️ Equalizer**: Audio frequency adjustment
- **📊 Statistics**: Play counts and listening history
- **🌐 Last.fm Integration**: Scrobbling support
- **🎧 Audio Effects**: Reverb, echo, and other effects
- **📱 Remote Control**: HTTP API for external control
- **🎵 Smart Playlists**: Auto-generated playlists based on criteria
- **🔍 Advanced Search**: Search by genre, year, album, etc.
- **🎨 Themes**: Customizable color schemes and UI themes

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Commit: `git commit -am 'Add feature'`
5. Push: `git push origin feature-name`
6. Create a Pull Request

### Code Style

- Follow Rust standard formatting: `cargo fmt`
- Ensure no warnings: `cargo clippy`
- Add tests for new features
- Update documentation

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Ratatui Team**: For the excellent TUI framework
- **Rodio Contributors**: For the robust audio library
- **Rust Community**: For the amazing ecosystem

## Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/uy-td-dev/LoFi-Turtle/issues)
- 📧 **Contact**: uy.td.dev@gmail.com

---

**Enjoy your music! 🎵**
