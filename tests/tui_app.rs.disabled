use wzllama::tui::App;
use wzllama::Screen;
use wzllama::tui::Navigation;

#[test]
fn test_app_initial_state() {
    let app = App::new_test();
    
    // État initial
    assert_eq!(app.current_screen, Screen::Information);
    assert!(app.sidebar_focus);
    assert!(app.selected_tool.is_none());
    assert!(!app.should_quit);
}

#[test]
fn test_app_move_down_sidebar() {
    let mut app = App::new_test();
    
    // Navigation dans le sidebar
    assert_eq!(app.current_screen, Screen::Information);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Models);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Tools);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Terminal);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Cleanup);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Config);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Language);
    
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Quit);
    
    // Circulaire
    app.navigate(Navigation::Down);
    assert_eq!(app.current_screen, Screen::Information);
}

#[test]
fn test_app_move_up_sidebar() {
    let mut app = App::new_test();
    
    // Navigation inverse dans le sidebar
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Quit);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Language);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Config);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Cleanup);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Terminal);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Tools);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Models);
    
    app.navigate(Navigation::Up);
    assert_eq!(app.current_screen, Screen::Information);
}

#[test]
fn test_app_tab_toggle_focus() {
    let mut app = App::new_test();
    
    // Naviguer vers Models d'abord
    app.navigate(Navigation::Down); // Information → Models
    
    // Initialement en focus sidebar
    assert!(app.sidebar_focus);
    assert!(app.selected_tool.is_none());
    
    // Tab pour basculer vers content
    app.navigate(Navigation::Search); // Tab
    assert!(!app.sidebar_focus);
    assert_eq!(app.selected_tool, Some(0));
    
    // Tab pour revenir au sidebar
    app.navigate(Navigation::Search);
    assert!(app.sidebar_focus);
    assert!(app.selected_tool.is_none());
}

#[test]
fn test_app_quit_from_main() {
    let mut app = App::new_test();
    
    // Quit (Esc) sur Information doit quitter
    app.navigate(Navigation::Quit); // Esc
    assert!(app.should_quit);
}

#[test]
fn test_app_left_on_information() {
    let mut app = App::new_test();
    
    // Left sur Information ne change pas d'écran (pas d'action)
    app.navigate(Navigation::Left); // ← sur Information
    assert_eq!(app.current_screen, Screen::Information);
    assert!(app.sidebar_focus);
}

#[test]
fn test_app_esc_returns_to_main() {
    // Esc depuis n'importe quel sous-menu retourne à Information (parent)
    
    // Test 1: Esc sur Models → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 2: Esc sur Tools → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 3: Esc sur Terminal → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 4: Esc sur Cleanup → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Down); // Terminal → Cleanup
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 5: Esc sur Config → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Down); // Terminal → Cleanup
    app.navigate(Navigation::Down); // Cleanup → Config
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 6: Esc sur Language → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Down); // Terminal → Cleanup
    app.navigate(Navigation::Down); // Cleanup → Config
    app.navigate(Navigation::Down); // Config → Language
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
    
    // Test 7: Esc sur Quit → Information
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Down); // Terminal → Cleanup
    app.navigate(Navigation::Down); // Cleanup → Config
    app.navigate(Navigation::Down); // Config → Language
    app.navigate(Navigation::Down); // Language → Quit
    app.navigate(Navigation::Left); // Esc
    assert_eq!(app.current_screen, Screen::Information);
}

#[test]
fn test_app_esc_from_content_mode() {
    // Models screen - Right on Models does NOT enter content mode currently
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Right); // Enter sur Models - stays on sidebar
    assert!(app.sidebar_focus); // Models doesn't enter content mode on select
    assert_eq!(app.current_screen, Screen::Models);
    
    // Terminal screen - enters content mode now (integrated terminal)
    let mut app = App::new_test();
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools  
    app.navigate(Navigation::Down); // Tools → Terminal
    app.navigate(Navigation::Right); // Enter sur Terminal - enters content mode
    assert!(!app.should_quit); // No longer quits immediately
    assert!(app.sidebar_focus == false); // Enters content mode
    assert_eq!(app.exec_command, None); // No exec command set
    
    // Test returning from content mode via Esc
    app.navigate(Navigation::Left); // Esc
    assert!(app.sidebar_focus); // Returns to sidebar focus
}

#[test]
fn test_app_select_models_enters_content_mode() {
    let mut app = App::new_test();
    
    // Naviguer vers Tools (qui entre en content mode)
    app.navigate(Navigation::Down); // Information → Models
    app.navigate(Navigation::Down); // Models → Tools
    app.navigate(Navigation::Right); // Enter sur Tools
    assert!(!app.sidebar_focus); // Tools enters content mode
    assert_eq!(app.selected_tool, Some(0));
}