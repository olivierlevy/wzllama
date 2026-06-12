use wzllama::config::i18n;

#[test]
fn test_global_reload_updates_current() {
    // Initialize global store
    i18n::init_global();

    // Reload to English explicitly
    i18n::reload("en").expect("reload en");
    let current = i18n::get_current();
    assert_eq!(current.meta.code, "en");

    // Reload to French explicitly
    i18n::reload("fr").expect("reload fr");
    let current2 = i18n::get_current();
    assert_eq!(current2.meta.code, "fr");
}
