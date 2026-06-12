use wzllama::wizard::menu_wizard::UseCase;

#[test]
fn test_usecase_all() {
    let all = UseCase::all();
    assert_eq!(all.len(), 6);
    assert!(all.contains(&UseCase::General));
    assert!(all.contains(&UseCase::Coding));
    assert!(all.contains(&UseCase::Reasoning));
    assert!(all.contains(&UseCase::Chat));
    assert!(all.contains(&UseCase::Multimodal));
    assert!(all.contains(&UseCase::Embedding));
}

#[test]
fn test_usecase_as_str() {
    assert_eq!(UseCase::General.as_str(), "general");
    assert_eq!(UseCase::Coding.as_str(), "coding");
    assert_eq!(UseCase::Reasoning.as_str(), "reasoning");
    assert_eq!(UseCase::Chat.as_str(), "chat");
    assert_eq!(UseCase::Multimodal.as_str(), "multimodal");
    assert_eq!(UseCase::Embedding.as_str(), "embedding");
}

#[test]
fn test_usecase_equality() {
    let uc1 = UseCase::Coding;
    let uc2 = UseCase::Coding;
    let uc3 = UseCase::Chat;

    assert_eq!(uc1, uc2);
    assert_ne!(uc1, uc3);
}
