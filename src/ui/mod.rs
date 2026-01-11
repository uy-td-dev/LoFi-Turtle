pub mod app;
pub mod widgets;
pub mod layout;
pub mod theme;

pub use app::{App, InputMode, ActivePanel, ViewMode};
pub use widgets::draw_ui;
// Re-exporting these for convenience, even if not all are used in every module
#[allow(unused_imports)]
pub use layout::{WidgetConfig, WidgetType, Position, SizeConstraint};
#[allow(unused_imports)]
pub use theme::{ThemeManager, ColorPalette, Themes};
