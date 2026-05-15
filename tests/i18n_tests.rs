use wzllama::config::i18n::{self, I18n, LanguageMeta};

#[test]
fn test_i18n_default() {
    let i18n = I18n::default();
    assert_eq!(i18n.meta.code, "en");
    assert_eq!(i18n.meta.name, "English");
    assert_eq!(i18n.meta.direction, "ltr");
}

#[test]
fn test_i18n_t_missing_key() {
    let i18n = I18n::default();
    // Les clés manquantes retournent la clé elle-même
    assert_eq!(i18n.t("missing.key"), "missing.key");
    assert_eq!(i18n.t("another.missing"), "another.missing");
}

#[test]
fn test_i18n_t_with_vars() {
    // Test avec une clé manquante
    let i18n = I18n::default();
    
    // Test sans variables dans la chaîne
    assert_eq!(i18n.t_with_vars("missing", &[]), "missing");
    
    // Test avec variable mais clé manquante - les {} sont remplacés
    // "missing.{name}" devient "missing.World" car le pattern {name} est remplacé
    let result = i18n.t_with_vars("missing.{name}", &[("name", "World")]);
    assert_eq!(result, "missing.World");
}

#[test]
fn test_i18n_t_with_vars_multiple() {
    // Test le comportement de t_with_vars avec une clé existante
    let i18n = I18n::default();
    
    // La clé n'existe pas, donc retourne la clé telle quelle
    let result = i18n.t_with_vars("greeting", &[("name", "World")]);
    assert_eq!(result, "greeting");
}

#[test]
fn test_language_meta_creation() {
    let lang = LanguageMeta {
        code: "fr".into(),
        name: "Français".into(),
        name_en: Some("French".into()),
        direction: "ltr".into(),
    };
    
    assert_eq!(lang.code, "fr");
    assert_eq!(lang.name, "Français");
    assert_eq!(lang.name_en, Some("French".into()));
}

#[test]
fn test_load_french_i18n() {
    // Test de chargement du fichier fr.json embarqué
    let result = i18n::load("fr");
    assert!(result.is_ok());
    let i18n = result.unwrap();
    assert_eq!(i18n.meta.code, "fr");
}

#[test]
fn test_load_english_i18n() {
    let result = i18n::load("en");
    assert!(result.is_ok());
    let i18n = result.unwrap();
    assert_eq!(i18n.meta.code, "en");
}

#[test]
fn test_load_missing_language() {
    let result = i18n::load("nonexistent");
    // Devrait retomber sur le fallback
    assert!(result.is_ok());
}

#[test]
fn test_detect_system_language() {
    let lang = i18n::detect_system_language();
    // Retourne une langue valide
    assert!(!lang.is_empty());
}

#[test]
fn test_get_available_languages() {
    let langs = i18n::get_available_languages();
    // Doit toujours retourner au moins une langue
    assert!(!langs.is_empty());
    
    // Vérifie que les langues ont les champs requis
    for lang in &langs {
        assert!(!lang.code.is_empty());
        assert!(!lang.name.is_empty());
    }
}