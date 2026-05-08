use thiserror::Error;

#[derive(Error, Debug)]
pub enum WzllamaError {
    #[error("Template {file} invalide: {message}")]
    InvalidTemplate { file: String, message: String },

    #[error("Installation de {tool} échouée")]
    InstallationFailed { tool: String },

    #[error("Outil non trouvé: {tool}")]
    ToolNotFound { tool: String },

    #[error("Erreur réseau: {0}")]
    NetworkError(String),

    #[error("Erreur de permission")]
    PermissionError,

    #[error("Annulé par l'utilisateur")]
    UserCancelled,

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for WzllamaError {
    fn from(e: anyhow::Error) -> Self {
        WzllamaError::Other(e.to_string())
    }
}