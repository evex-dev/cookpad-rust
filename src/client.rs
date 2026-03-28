// Python の client.py に対応する HTTP クライアント実装。
//
// 非同期 (tokio + reqwest) で実装する。
// Python 版の async with Cookpad() as client: に対応して、
// Cookpad::new() でクライアントを作成し、各メソッドを await で呼ぶ。

use reqwest::{header, Client};
use serde_json::Value;
use uuid::Uuid;

use crate::constants::*;
use crate::error::{CookpadError, Result};
use crate::types::*;

// ------------------------------------------------------------
// クライアント構造体
// Python: class Cookpad の __init__ 引数に対応
// ------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Cookpad {
    token: String,
    country: String,
    language: String,
    timezone_id: String,
    timezone_offset: String,
    user_agent: String,
    provider_id: String,
    client: Client,
}

impl Cookpad {
    /// デフォルト設定でクライアントを作成する。
    /// Python: Cookpad() に対応
    pub fn new() -> Self {
        Self::with_config(
            DEFAULT_TOKEN,
            DEFAULT_COUNTRY,
            DEFAULT_LANGUAGE,
            DEFAULT_TIMEZONE_ID,
            DEFAULT_TIMEZONE_OFFSET,
            DEFAULT_USER_AGENT,
            DEFAULT_PROVIDER_ID,
        )
    }

    /// カスタム設定でクライアントを作成する。
    /// Python: Cookpad(token=..., country=..., ...) に対応
    pub fn with_config(
        token: impl Into<String>,
        country: impl Into<String>,
        language: impl Into<String>,
        timezone_id: impl Into<String>,
        timezone_offset: impl Into<String>,
        user_agent: impl Into<String>,
        provider_id: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            country: country.into(),
            language: language.into(),
            timezone_id: timezone_id.into(),
            timezone_offset: timezone_offset.into(),
            user_agent: user_agent.into(),
            provider_id: provider_id.into(),
            client: Client::builder()
                .gzip(true)
                .build()
                .expect("Failed to build reqwest client"),
        }
    }

    /// リクエストヘッダーを構築する。
    /// Python: Cookpad._headers() に対応
    fn headers(&self) -> header::HeaderMap {
        let guid = Uuid::new_v4().to_string().to_uppercase();
        let mut map = header::HeaderMap::new();

        let insert = |map: &mut header::HeaderMap, key: &str, val: &str| {
            map.insert(
                header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                header::HeaderValue::from_str(val).unwrap(),
            );
        };

        insert(&mut map, "Host",                          API_HOST);
        insert(&mut map, "Authorization",                 &format!("Bearer {}", self.token));
        insert(&mut map, "X-Cookpad-Country-Selected",    &self.country);
        insert(&mut map, "X-Cookpad-Timezone-Id",         &self.timezone_id);
        insert(&mut map, "X-Cookpad-Provider-Id",         &self.provider_id);
        insert(&mut map, "X-Cookpad-Timezone-Offset",     &self.timezone_offset);
        insert(&mut map, "X-Cookpad-Guid",                &guid);
        insert(&mut map, "Accept-Encoding",               "gzip");
        insert(&mut map, "Accept-Language",               &self.language);
        insert(&mut map, "Accept",                        "*/*");
        insert(&mut map, "User-Agent",                    &self.user_agent);

        map
    }

    /// HTTP GET リクエストを実行して JSON を返す内部メソッド。
    /// Python: Cookpad._request() に対応
    async fn request(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<Value> {
        let url = format!("{}{}", BASE_URL, path);

        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(params)
            .send()
            .await?;

        // ステータスコードに応じてエラーを返す
        // Python: Cookpad._request() のステータス分岐に対応
        match resp.status().as_u16() {
            401 => return Err(CookpadError::AuthenticationError),
            404 => return Err(CookpadError::NotFoundError(path.to_string())),
            429 => return Err(CookpadError::RateLimitError),
            s if s >= 400 => {
                let msg = resp.text().await.unwrap_or_default();
                return Err(CookpadError::ApiError { status: s, message: msg });
            }
            _ => {}
        }

        Ok(resp.json().await?)
    }

    // ----------------------------------------------------------
    // レシピ検索
    // Python: Cookpad.search_recipes() に対応
    // ----------------------------------------------------------
    pub async fn search_recipes(
        &self,
        query: &str,
    ) -> Result<SearchResponse> {
        self.search_recipes_with_options(query, 1, 30).await
    }

    pub async fn search_recipes_with_options(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<SearchResponse> {
        let page_s = page.to_string();
        let per_page_s = per_page.to_string();

        let params = [
            ("query",                       query),
            ("page",                        &page_s),
            ("per_page",                    &per_page_s),
            ("order",                       "recent"),
            ("must_have_cooksnaps",         "false"),
            ("minimum_number_of_cooksnaps", "0"),
            ("must_have_photo_in_steps",    "false"),
            ("from_delicious_ways",         "false"),
            ("search_source",               "recipe.search.typed_query"),
            ("supported_types",             SUPPORTED_SEARCH_TYPES),
        ];

        let data = self.request("/search_results", &params).await?;
        Ok(serde_json::from_value(data)?)
    }

    // ----------------------------------------------------------
    // レシピ詳細取得
    // Python: Cookpad.get_recipe() に対応
    // ----------------------------------------------------------
    pub async fn get_recipe(&self, recipe_id: u64) -> Result<Recipe> {
        let path = format!("/recipes/{}", recipe_id);
        let data = self.request(&path, &[]).await?;
        let result = data.get("result").cloned().unwrap_or(Value::Null);
        Ok(serde_json::from_value(result)?)
    }

    // ----------------------------------------------------------
    // 類似レシピ取得
    // Python: Cookpad.get_similar_recipes() に対応
    // ----------------------------------------------------------
    pub async fn get_similar_recipes(
        &self,
        recipe_id: u64,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<Recipe>> {
        let path = format!("/recipes/{}/similar_recipes", recipe_id);
        let page_s = page.to_string();
        let per_page_s = per_page.to_string();
        let params = [("page", page_s.as_str()), ("per_page", per_page_s.as_str())];
        let data = self.request(&path, &params).await?;

        let recipes = data
            .get("result")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(recipes)
    }

    // ----------------------------------------------------------
    // コメント取得
    // Python: Cookpad.get_comments() に対応
    // ----------------------------------------------------------
    pub async fn get_comments(
        &self,
        recipe_id: u64,
        limit: u32,
        after: &str,
        label: &str,
    ) -> Result<CommentsResponse> {
        let path = format!("/recipes/{}/comments", recipe_id);
        let limit_s = limit.to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("limit", &limit_s),
            ("label", label),
        ];
        if !after.is_empty() {
            params.push(("after", after));
        }
        let data = self.request(&path, &params).await?;
        Ok(serde_json::from_value(data)?)
    }

    // ----------------------------------------------------------
    // ユーザー検索
    // Python: Cookpad.search_users() に対応
    // ----------------------------------------------------------
    pub async fn search_users(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<UsersResponse> {
        let page_s = page.to_string();
        let per_page_s = per_page.to_string();
        let params = [
            ("query",    query),
            ("page",     &page_s),
            ("per_page", &per_page_s),
        ];
        let data = self.request("/users", &params).await?;
        Ok(serde_json::from_value(data)?)
    }

    // ----------------------------------------------------------
    // 検索キーワードサジェスト
    // Python: Cookpad.search_keywords() に対応
    // ----------------------------------------------------------
    pub async fn search_keywords(&self, query: &str) -> Result<Value> {
        let params = if query.is_empty() { vec![] } else { vec![("query", query)] };
        let data = self.request("/search_keywords", &params).await?;
        Ok(data.get("result").cloned().unwrap_or(Value::Object(Default::default())))
    }

    // ----------------------------------------------------------
    // 検索履歴・トレンドキーワード
    // Python: Cookpad.get_search_history() に対応
    // ----------------------------------------------------------
    pub async fn get_search_history(&self, local_history: &[&str]) -> Result<Value> {
        let history_json = serde_json::to_string(local_history)
            .unwrap_or_else(|_| "[]".to_string());
        let params = [("local_search_history", history_json.as_str())];
        self.request("/search_history", &params).await
    }
}

impl Default for Cookpad {
    fn default() -> Self {
        Self::new()
    }
}
