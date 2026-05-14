use wzllama::Screen;

#[test]
fn test_screen_menu_order() {
    // Vérifie l'ordre des écrans dans le menu sidebar
    let menu_screens = Screen::menu_screens();
    assert_eq!(menu_screens.len(), 8);
    
    // Ordre attendu (comme dans le wizard menu_main.rs)
    assert_eq!(menu_screens[0], Screen::Information);
    assert_eq!(menu_screens[1], Screen::Models);
    assert_eq!(menu_screens[2], Screen::Tools);
    assert_eq!(menu_screens[3], Screen::Terminal);
    assert_eq!(menu_screens[4], Screen::Cleanup);
    assert_eq!(menu_screens[5], Screen::Config);
    assert_eq!(menu_screens[6], Screen::Language);
    assert_eq!(menu_screens[7], Screen::Quit);
}

#[test]
fn test_screen_next_menu() {
    // Test la navigation circulaire dans le menu
    assert_eq!(Screen::Information.next_menu(), Screen::Models);
    assert_eq!(Screen::Models.next_menu(), Screen::Tools);
    assert_eq!(Screen::Tools.next_menu(), Screen::Terminal);
    assert_eq!(Screen::Terminal.next_menu(), Screen::Cleanup);
    assert_eq!(Screen::Cleanup.next_menu(), Screen::Config);
    assert_eq!(Screen::Config.next_menu(), Screen::Language);
    assert_eq!(Screen::Language.next_menu(), Screen::Quit);
    assert_eq!(Screen::Quit.next_menu(), Screen::Information); // Circulaire
}

#[test]
fn test_screen_prev_menu() {
    // Test la navigation circulaire inverse
    assert_eq!(Screen::Information.prev_menu(), Screen::Quit);
    assert_eq!(Screen::Quit.prev_menu(), Screen::Language);
    assert_eq!(Screen::Language.prev_menu(), Screen::Config);
    assert_eq!(Screen::Config.prev_menu(), Screen::Cleanup);
    assert_eq!(Screen::Cleanup.prev_menu(), Screen::Terminal);
    assert_eq!(Screen::Terminal.prev_menu(), Screen::Tools);
    assert_eq!(Screen::Tools.prev_menu(), Screen::Models);
    assert_eq!(Screen::Models.prev_menu(), Screen::Information); // Circulaire
}

#[test]
fn test_screen_titles() {
    // Test que chaque écran a un titre
    assert!(!Screen::Information.title().is_empty());
    assert!(!Screen::Models.title().is_empty());
    assert!(!Screen::Tools.title().is_empty());
    assert!(!Screen::Terminal.title().is_empty());
    assert!(!Screen::Cleanup.title().is_empty());
    assert!(!Screen::Config.title().is_empty());
    assert!(!Screen::Language.title().is_empty());
    assert!(!Screen::Quit.title().is_empty());
}