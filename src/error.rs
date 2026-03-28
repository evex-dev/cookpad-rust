// Python の exceptions.py に対応するエラー型。
// thiserror クレートで定義し、? 演算子で自動変換できるようにする。

#[derive(Debug, thiserror::Error)]
pub enum CookpadError {
    /// 認証失敗 (HTTP 401) — Python: AuthenticationError
    #[error("Authentication failed")]
    AuthenticationError,

    /// リソース未発見 (HTTP 404) — Python: NotFoundError
    #[error("Not found: {0}")]
    NotFoundError(String),

    /// レート制限 (HTTP 429) — Python: RateLimitError
    #[error("Rate limit exceeded")]
    RateLimitError,

    /// その他の API エラー (HTTP 4xx/5xx) — Python: APIError
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    /// ネットワーク・reqwest レベルのエラー
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    /// JSON パースエラー
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CookpadError>;
