use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Theme color palette for the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    
    // Base colors
    pub background: ColorSpec,
    pub foreground: ColorSpec,
    
    // UI Elements
    pub border: ColorSpec,
    pub border_focused: ColorSpec,
    pub title: ColorSpec,
    pub status_bar: ColorSpec,
    pub status_bar_mode: ColorSpec,
    
    // Text styles
    pub text_normal: ColorSpec,
    pub text_dim: ColorSpec,
    pub text_bold: ColorSpec,
    pub text_highlight: ColorSpec,
    
    // Interactive elements
    pub cursor: ColorSpec,
    pub selection: ColorSpec,
    pub visual_selection: ColorSpec,
    pub active_field: ColorSpec,
    pub insert_mode: ColorSpec,
    
    // Status indicators
    pub success: ColorSpec,
    pub warning: ColorSpec,
    pub error: ColorSpec,
    pub info: ColorSpec,
    
    // Email list
    pub email_from: ColorSpec,
    pub email_subject: ColorSpec,
    pub email_date: ColorSpec,
    pub email_unread: ColorSpec,
    
    // Compose view
    pub compose_field_label: ColorSpec,
    pub compose_field_value: ColorSpec,
    pub compose_field_empty: ColorSpec,
    
    // Markdown preview
    pub markdown_heading: ColorSpec,
    pub markdown_emphasis: ColorSpec,
    pub markdown_link: ColorSpec,
    pub markdown_code: ColorSpec,
}

/// Color specification that can be serialized/deserialized
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorSpec {
    /// Named color (e.g., "red", "blue", "cyan")
    Named(String),
    /// RGB color (r, g, b)
    Rgb(u8, u8, u8),
    /// Indexed color (0-255)
    Indexed(u8),
}

impl ColorSpec {
    pub fn to_color(&self) -> Color {
        match self {
            ColorSpec::Named(name) => Self::parse_named_color(name),
            ColorSpec::Rgb(r, g, b) => Color::Rgb(*r, *g, *b),
            ColorSpec::Indexed(i) => Color::Indexed(*i),
        }
    }
    
    fn parse_named_color(name: &str) -> Color {
        match name.to_lowercase().as_str() {
            "reset" => Color::Reset,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            _ => {
                // Log warning for unrecognized color names to help debug config issues
                eprintln!("Warning: Unrecognized color name '{}', defaulting to Reset", name);
                Color::Reset
            }
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::gruvbox_dark()
    }
}

impl Theme {
    /// Gruvbox Dark theme (default)
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark".to_string(),
            
            // Base colors
            background: ColorSpec::Rgb(40, 40, 40),      // #282828
            foreground: ColorSpec::Rgb(235, 219, 178),   // #ebdbb2
            
            // UI Elements
            border: ColorSpec::Rgb(146, 131, 116),       // #928374
            border_focused: ColorSpec::Rgb(254, 128, 25), // #fe8019 (bright orange)
            title: ColorSpec::Rgb(142, 192, 124),        // #8ec07c (aqua)
            status_bar: ColorSpec::Rgb(60, 56, 54),      // #3c3836
            status_bar_mode: ColorSpec::Rgb(251, 241, 199), // #fbf1c7 (fg0)
            
            // Text styles
            text_normal: ColorSpec::Rgb(235, 219, 178),  // #ebdbb2
            text_dim: ColorSpec::Rgb(146, 131, 116),     // #928374
            text_bold: ColorSpec::Rgb(251, 241, 199),    // #fbf1c7
            text_highlight: ColorSpec::Rgb(250, 189, 47), // #fabd2f (yellow)
            
            // Interactive elements
            cursor: ColorSpec::Rgb(80, 73, 69),          // #504945
            selection: ColorSpec::Rgb(69, 133, 136),     // #458588 (blue)
            visual_selection: ColorSpec::Rgb(102, 92, 84), // #665c54
            active_field: ColorSpec::Rgb(142, 192, 124), // #8ec07c (aqua)
            insert_mode: ColorSpec::Rgb(250, 189, 47),   // #fabd2f (yellow)
            
            // Status indicators
            success: ColorSpec::Rgb(184, 187, 38),       // #b8bb26 (green)
            warning: ColorSpec::Rgb(250, 189, 47),       // #fabd2f (yellow)
            error: ColorSpec::Rgb(251, 73, 52),          // #fb4934 (red)
            info: ColorSpec::Rgb(131, 165, 152),         // #83a598 (blue)
            
            // Email list
            email_from: ColorSpec::Rgb(142, 192, 124),   // #8ec07c (aqua)
            email_subject: ColorSpec::Rgb(235, 219, 178), // #ebdbb2
            email_date: ColorSpec::Rgb(146, 131, 116),   // #928374
            email_unread: ColorSpec::Rgb(251, 241, 199), // #fbf1c7 (bold fg)
            
            // Compose view
            compose_field_label: ColorSpec::Rgb(142, 192, 124), // #8ec07c (aqua)
            compose_field_value: ColorSpec::Rgb(235, 219, 178), // #ebdbb2
            compose_field_empty: ColorSpec::Rgb(146, 131, 116), // #928374
            
            // Markdown preview
            markdown_heading: ColorSpec::Rgb(250, 189, 47),     // #fabd2f (yellow)
            markdown_emphasis: ColorSpec::Rgb(254, 128, 25),    // #fe8019 (orange)
            markdown_link: ColorSpec::Rgb(131, 165, 152),       // #83a598 (blue)
            markdown_code: ColorSpec::Rgb(184, 187, 38),        // #b8bb26 (green)
        }
    }
    
    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            
            // Base colors
            background: ColorSpec::Rgb(40, 42, 54),      // #282a36
            foreground: ColorSpec::Rgb(248, 248, 242),   // #f8f8f2
            
            // UI Elements
            border: ColorSpec::Rgb(98, 114, 164),        // #6272a4 (comment)
            border_focused: ColorSpec::Rgb(255, 121, 198), // #ff79c6 (pink)
            title: ColorSpec::Rgb(139, 233, 253),        // #8be9fd (cyan)
            status_bar: ColorSpec::Rgb(68, 71, 90),      // #44475a (selection)
            status_bar_mode: ColorSpec::Rgb(248, 248, 242), // #f8f8f2
            
            // Text styles
            text_normal: ColorSpec::Rgb(248, 248, 242),  // #f8f8f2
            text_dim: ColorSpec::Rgb(98, 114, 164),      // #6272a4
            text_bold: ColorSpec::Rgb(255, 255, 255),    // white
            text_highlight: ColorSpec::Rgb(241, 250, 140), // #f1fa8c (yellow)
            
            // Interactive elements
            cursor: ColorSpec::Rgb(68, 71, 90),          // #44475a
            selection: ColorSpec::Rgb(68, 71, 90),       // #44475a
            visual_selection: ColorSpec::Rgb(98, 114, 164), // #6272a4
            active_field: ColorSpec::Rgb(139, 233, 253), // #8be9fd (cyan)
            insert_mode: ColorSpec::Rgb(241, 250, 140),  // #f1fa8c (yellow)
            
            // Status indicators
            success: ColorSpec::Rgb(80, 250, 123),       // #50fa7b (green)
            warning: ColorSpec::Rgb(241, 250, 140),      // #f1fa8c (yellow)
            error: ColorSpec::Rgb(255, 85, 85),          // #ff5555 (red)
            info: ColorSpec::Rgb(139, 233, 253),         // #8be9fd (cyan)
            
            // Email list
            email_from: ColorSpec::Rgb(139, 233, 253),   // #8be9fd (cyan)
            email_subject: ColorSpec::Rgb(248, 248, 242), // #f8f8f2
            email_date: ColorSpec::Rgb(98, 114, 164),    // #6272a4
            email_unread: ColorSpec::Rgb(255, 121, 198), // #ff79c6 (pink)
            
            // Compose view
            compose_field_label: ColorSpec::Rgb(139, 233, 253), // #8be9fd (cyan)
            compose_field_value: ColorSpec::Rgb(248, 248, 242), // #f8f8f2
            compose_field_empty: ColorSpec::Rgb(98, 114, 164),  // #6272a4
            
            // Markdown preview
            markdown_heading: ColorSpec::Rgb(255, 121, 198),    // #ff79c6 (pink)
            markdown_emphasis: ColorSpec::Rgb(189, 147, 249),   // #bd93f9 (purple)
            markdown_link: ColorSpec::Rgb(139, 233, 253),       // #8be9fd (cyan)
            markdown_code: ColorSpec::Rgb(80, 250, 123),        // #50fa7b (green)
        }
    }
    
    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            
            // Base colors
            background: ColorSpec::Rgb(46, 52, 64),      // #2e3440
            foreground: ColorSpec::Rgb(236, 239, 244),   // #eceff4
            
            // UI Elements
            border: ColorSpec::Rgb(76, 86, 106),         // #4c566a
            border_focused: ColorSpec::Rgb(136, 192, 208), // #88c0d0 (frost 1)
            title: ColorSpec::Rgb(143, 188, 187),        // #8fbcbb (frost 0)
            status_bar: ColorSpec::Rgb(59, 66, 82),      // #3b4252
            status_bar_mode: ColorSpec::Rgb(236, 239, 244), // #eceff4
            
            // Text styles
            text_normal: ColorSpec::Rgb(236, 239, 244),  // #eceff4
            text_dim: ColorSpec::Rgb(76, 86, 106),       // #4c566a
            text_bold: ColorSpec::Rgb(236, 239, 244),    // #eceff4
            text_highlight: ColorSpec::Rgb(235, 203, 139), // #ebcb8b (aurora yellow)
            
            // Interactive elements
            cursor: ColorSpec::Rgb(67, 76, 94),          // #434c5e
            selection: ColorSpec::Rgb(94, 129, 172),     // #5e81ac (frost 3)
            visual_selection: ColorSpec::Rgb(76, 86, 106), // #4c566a
            active_field: ColorSpec::Rgb(143, 188, 187), // #8fbcbb (frost 0)
            insert_mode: ColorSpec::Rgb(235, 203, 139),  // #ebcb8b (aurora yellow)
            
            // Status indicators
            success: ColorSpec::Rgb(163, 190, 140),      // #a3be8c (aurora green)
            warning: ColorSpec::Rgb(235, 203, 139),      // #ebcb8b (aurora yellow)
            error: ColorSpec::Rgb(191, 97, 106),         // #bf616a (aurora red)
            info: ColorSpec::Rgb(136, 192, 208),         // #88c0d0 (frost 1)
            
            // Email list
            email_from: ColorSpec::Rgb(143, 188, 187),   // #8fbcbb (frost 0)
            email_subject: ColorSpec::Rgb(236, 239, 244), // #eceff4
            email_date: ColorSpec::Rgb(76, 86, 106),     // #4c566a
            email_unread: ColorSpec::Rgb(229, 233, 240), // #e5e9f0 (snow 1)
            
            // Compose view
            compose_field_label: ColorSpec::Rgb(143, 188, 187), // #8fbcbb (frost 0)
            compose_field_value: ColorSpec::Rgb(236, 239, 244), // #eceff4
            compose_field_empty: ColorSpec::Rgb(76, 86, 106),   // #4c566a
            
            // Markdown preview
            markdown_heading: ColorSpec::Rgb(136, 192, 208),    // #88c0d0 (frost 1)
            markdown_emphasis: ColorSpec::Rgb(180, 142, 173),   // #b48ead (aurora purple)
            markdown_link: ColorSpec::Rgb(94, 129, 172),        // #5e81ac (frost 3)
            markdown_code: ColorSpec::Rgb(163, 190, 140),       // #a3be8c (aurora green)
        }
    }
    
    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            
            // Base colors
            background: ColorSpec::Rgb(0, 43, 54),       // #002b36
            foreground: ColorSpec::Rgb(131, 148, 150),   // #839496
            
            // UI Elements
            border: ColorSpec::Rgb(88, 110, 117),        // #586e75
            border_focused: ColorSpec::Rgb(38, 139, 210), // #268bd2 (blue)
            title: ColorSpec::Rgb(42, 161, 152),         // #2aa198 (cyan)
            status_bar: ColorSpec::Rgb(7, 54, 66),       // #073642
            status_bar_mode: ColorSpec::Rgb(238, 232, 213), // #eee8d5
            
            // Text styles
            text_normal: ColorSpec::Rgb(131, 148, 150),  // #839496
            text_dim: ColorSpec::Rgb(88, 110, 117),      // #586e75
            text_bold: ColorSpec::Rgb(238, 232, 213),    // #eee8d5
            text_highlight: ColorSpec::Rgb(181, 137, 0), // #b58900 (yellow)
            
            // Interactive elements
            cursor: ColorSpec::Rgb(7, 54, 66),           // #073642
            selection: ColorSpec::Rgb(38, 139, 210),     // #268bd2 (blue)
            visual_selection: ColorSpec::Rgb(88, 110, 117), // #586e75
            active_field: ColorSpec::Rgb(42, 161, 152),  // #2aa198 (cyan)
            insert_mode: ColorSpec::Rgb(181, 137, 0),    // #b58900 (yellow)
            
            // Status indicators
            success: ColorSpec::Rgb(133, 153, 0),        // #859900 (green)
            warning: ColorSpec::Rgb(203, 75, 22),        // #cb4b16 (orange)
            error: ColorSpec::Rgb(220, 50, 47),          // #dc322f (red)
            info: ColorSpec::Rgb(108, 113, 196),         // #6c71c4 (violet)
            
            // Email list
            email_from: ColorSpec::Rgb(42, 161, 152),    // #2aa198 (cyan)
            email_subject: ColorSpec::Rgb(131, 148, 150), // #839496
            email_date: ColorSpec::Rgb(88, 110, 117),    // #586e75
            email_unread: ColorSpec::Rgb(238, 232, 213), // #eee8d5
            
            // Compose view
            compose_field_label: ColorSpec::Rgb(42, 161, 152), // #2aa198 (cyan)
            compose_field_value: ColorSpec::Rgb(131, 148, 150), // #839496
            compose_field_empty: ColorSpec::Rgb(88, 110, 117),  // #586e75
            
            // Markdown preview
            markdown_heading: ColorSpec::Rgb(203, 75, 22),      // #cb4b16 (orange)
            markdown_emphasis: ColorSpec::Rgb(211, 54, 130),    // #d33682 (magenta)
            markdown_link: ColorSpec::Rgb(38, 139, 210),        // #268bd2 (blue)
            markdown_code: ColorSpec::Rgb(133, 153, 0),         // #859900 (green)
        }
    }
    
    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            
            // Base colors
            background: ColorSpec::Rgb(26, 27, 38),      // #1a1b26
            foreground: ColorSpec::Rgb(192, 202, 245),   // #c0caf5
            
            // UI Elements
            border: ColorSpec::Rgb(68, 75, 106),         // #444b6a
            border_focused: ColorSpec::Rgb(125, 207, 255), // #7dcfff (cyan)
            title: ColorSpec::Rgb(125, 207, 255),        // #7dcfff (cyan)
            status_bar: ColorSpec::Rgb(36, 40, 59),      // #24283b
            status_bar_mode: ColorSpec::Rgb(192, 202, 245), // #c0caf5
            
            // Text styles
            text_normal: ColorSpec::Rgb(192, 202, 245),  // #c0caf5
            text_dim: ColorSpec::Rgb(68, 75, 106),       // #444b6a
            text_bold: ColorSpec::Rgb(192, 202, 245),    // #c0caf5
            text_highlight: ColorSpec::Rgb(224, 175, 104), // #e0af68 (yellow)
            
            // Interactive elements
            cursor: ColorSpec::Rgb(52, 59, 88),          // #343b58
            selection: ColorSpec::Rgb(42, 195, 222),     // #2ac3de (teal)
            visual_selection: ColorSpec::Rgb(68, 75, 106), // #444b6a
            active_field: ColorSpec::Rgb(125, 207, 255), // #7dcfff (cyan)
            insert_mode: ColorSpec::Rgb(224, 175, 104),  // #e0af68 (yellow)
            
            // Status indicators
            success: ColorSpec::Rgb(158, 206, 106),      // #9ece6a (green)
            warning: ColorSpec::Rgb(224, 175, 104),      // #e0af68 (yellow)
            error: ColorSpec::Rgb(247, 118, 142),        // #f7768e (red)
            info: ColorSpec::Rgb(125, 207, 255),         // #7dcfff (cyan)
            
            // Email list
            email_from: ColorSpec::Rgb(125, 207, 255),   // #7dcfff (cyan)
            email_subject: ColorSpec::Rgb(192, 202, 245), // #c0caf5
            email_date: ColorSpec::Rgb(68, 75, 106),     // #444b6a
            email_unread: ColorSpec::Rgb(192, 202, 245), // #c0caf5
            
            // Compose view
            compose_field_label: ColorSpec::Rgb(125, 207, 255), // #7dcfff (cyan)
            compose_field_value: ColorSpec::Rgb(192, 202, 245), // #c0caf5
            compose_field_empty: ColorSpec::Rgb(68, 75, 106),   // #444b6a
            
            // Markdown preview
            markdown_heading: ColorSpec::Rgb(187, 154, 247),    // #bb9af7 (purple)
            markdown_emphasis: ColorSpec::Rgb(255, 158, 100),   // #ff9e64 (orange)
            markdown_link: ColorSpec::Rgb(125, 207, 255),       // #7dcfff (cyan)
            markdown_code: ColorSpec::Rgb(158, 206, 106),       // #9ece6a (green)
        }
    }
    
    /// Get a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "gruvbox" | "gruvbox-dark" | "gruvbox_dark" => Some(Self::gruvbox_dark()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "solarized" | "solarized-dark" | "solarized_dark" => Some(Self::solarized_dark()),
            "tokyo-night" | "tokyo_night" | "tokyonight" => Some(Self::tokyo_night()),
            _ => None,
        }
    }
    
    /// Get all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "gruvbox-dark",
            "dracula",
            "nord",
            "solarized-dark",
            "tokyo-night",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("gruvbox").is_some());
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("nord").is_some());
        assert!(Theme::by_name("solarized-dark").is_some());
        assert!(Theme::by_name("tokyo-night").is_some());
        assert!(Theme::by_name("nonexistent").is_none());
    }
    
    #[test]
    fn test_color_spec_to_color() {
        let named = ColorSpec::Named("red".to_string());
        assert_eq!(named.to_color(), Color::Red);
        
        let rgb = ColorSpec::Rgb(255, 0, 0);
        assert_eq!(rgb.to_color(), Color::Rgb(255, 0, 0));
        
        let indexed = ColorSpec::Indexed(42);
        assert_eq!(indexed.to_color(), Color::Indexed(42));
    }
    
    #[test]
    fn test_default_theme_is_gruvbox() {
        let default = Theme::default();
        assert_eq!(default.name, "Gruvbox Dark");
    }
    
    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"gruvbox-dark"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.len() >= 5);
    }
}
