# 🐢 LofiTurtle

> **A chill, customizable terminal music player written in Rust.**
> *Code, Relax, Listen.*

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)

LofiTurtle là một trình phát nhạc TUI (Terminal User Interface) hiện đại, được thiết kế tối ưu cho hiệu năng và trải nghiệm người dùng. Với giao diện đẹp mắt, hỗ trợ theme và khả năng tùy biến layout mạnh mẽ, LofiTurtle mang đến không gian nghe nhạc thư giãn ngay trong terminal của bạn.

---

## ✨ Tính Năng Nổi Bật

*   **🎨 Giao Diện Hiện Đại:** Sử dụng **Rounded Borders** (viền bo tròn), icon trực quan và bố cục thông minh.
*   **🌙 Lofi Night Theme:** Theme mặc định lấy cảm hứng từ Dracula (Tím/Hồng/Cyan) dịu mắt, phù hợp cho coding ban đêm.
*   **🛠️ Tùy Biến Cao (Customizable):**
    *   Chỉnh sửa màu sắc, bố cục widget thông qua file `layout.toml`.
    *   Hỗ trợ Hot-reload: Sửa config, giao diện cập nhật ngay lập tức.
*   **🚀 Hiệu Năng Cao:**
    *   Backend **SQLite** quản lý thư viện nhạc lớn cực nhanh.
    *   Tối ưu hóa **Bulk Insert** giúp quét hàng nghìn bài hát trong tích tắc.
    *   Sử dụng Caching thông minh để giảm tải CPU khi render giao diện.
*   **🌊 Visualizer & Album Art:** Hiển thị ảnh bìa (dạng text/block) và sóng nhạc giả lập sống động.
*   **📂 Quản Lý Thư Viện:** Tìm kiếm nhanh (Fuzzy search), tạo Playlist, Shuffle thông minh (Fair randomization).

---

## 📥 Cài Đặt

### Yêu cầu
*   [Rust & Cargo](https://www.rust-lang.org/tools/install) (phiên bản mới nhất)

### Build từ source
```bash
# Clone repository
git clone https://github.com/uy-td-dev/lofiturtle.git
cd lofiturtle

# Build và chạy
cargo run --release
```

---

## 🎮 Hướng Dẫn Sử Dụng

Giao diện được chia thành 3 phần chính: **Header (Tìm kiếm)**, **Main Content (Playlist/Songs/Visuals)**, và **Player Controls**.

### Phím Tắt (Keybindings)

| Phím | Chức năng |
| :--- | :--- |
| **Điều khiển nhạc** | |
| `Space` | Phát / Tạm dừng (Play/Pause) |
| `n` | Bài tiếp theo (Next) |
| `p` | Bài trước đó (Previous) |
| `s` | Dừng hẳn (Stop) |
| `[` / `]` | Giảm / Tăng âm lượng |
| `S` (Shift+s) | Bật/Tắt Shuffle (Trộn bài) |
| `R` (Shift+r) | Đổi chế độ Repeat (Lặp lại) |
| **Điều hướng** | |
| `Tab` | Chuyển đổi giữa các bảng (Playlist <-> Songs) |
| `↑` / `↓` / `j` / `k` | Di chuyển lên xuống |
| `Enter` | Chọn bài hát / Mở Playlist |
| `Backspace` | Quay lại thư viện chính (All Songs) |
| **Tính năng khác** | |
| `/` | **Tìm kiếm** (Gõ tên bài, ca sĩ...) |
| `a` | Bật/Tắt Album Art & Visuals |
| `n` (tại Playlist) | Tạo Playlist mới |
| `d` (tại Playlist) | Xóa Playlist |
| `+` / `-` | Thêm/Xóa bài hát khỏi Playlist |
| `q` | Thoát ứng dụng |

---

## ⚙️ Tùy Biến (Customization)

LofiTurtle cho phép bạn tự do sáng tạo giao diện theo cá tính. File cấu hình thường nằm tại thư mục chạy ứng dụng hoặc bạn có thể chỉ định qua CLI.

### Cấu trúc `layout.toml`

Bạn có thể thay đổi màu sắc theo mã Hex để phù hợp với setup của mình:

```toml
[theme]
name = "my_custom_theme"

[theme.colors]
primary = "#bd93f9"      # Màu chính (Focus, Highlight)
secondary = "#ff79c6"    # Màu phụ (Icons, Accents)
background = "#282a36"   # Màu nền
foreground = "#f8f8f2"   # Màu chữ
border = "#6272a4"       # Màu viền
highlight = "#8be9fd"    # Màu khi đang chọn
error = "#ff5555"        # Màu lỗi
success = "#50fa7b"      # Màu thành công
```

Bạn cũng có thể ẩn/hiện các widget hoặc thay đổi vị trí của chúng trong phần `[[widgets]]`.

---

## 🛠️ Công Nghệ

Dự án được xây dựng dựa trên các thư viện Rust mạnh mẽ:
*   **[Ratatui](https://github.com/ratatui-org/ratatui):** Thư viện TUI cốt lõi để vẽ giao diện.
*   **[Rodio](https://github.com/RustAudio/rodio):** Xử lý âm thanh và playback.
*   **[Rusqlite](https://github.com/rusqlite/rusqlite):** Cơ sở dữ liệu SQLite nhúng.
*   **[Lofty](https://github.com/Serial-ATA/lofty-rs):** Đọc metadata và tag của file nhạc.

---

## 🤝 Đóng Góp

Mọi đóng góp đều được hoan nghênh! Nếu bạn tìm thấy lỗi hoặc muốn thêm tính năng mới, hãy mở Issue hoặc Pull Request.

1.  Fork dự án
2.  Tạo branch tính năng (`git checkout -b feature/AmazingFeature`)
3.  Commit thay đổi (`git commit -m 'Add some AmazingFeature'`)
4.  Push lên branch (`git push origin feature/AmazingFeature`)
5.  Mở Pull Request

---

## 📄 License

Được phân phối dưới giấy phép MIT. Xem `LICENSE` để biết thêm thông tin.

---

<p align="center">
  Made with ❤️ and 🦀 by <a href="https://github.com/uy-td-dev">Uy Tran</a>
</p>
