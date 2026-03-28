// Python の types.py (dataclass + parse_* 関数) に対応する型定義。
//
// serde の #[derive(Deserialize)] を使うことで、
// Python 版の parse_*() 関数群が不要になる。
// JSON フィールド名と Rust のフィールド名が異なる場合は
// #[serde(rename = "...")] または #[serde(rename_all = "snake_case")] で対応する。

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ------------------------------------------------------------
// Image — Python: class Image
// ------------------------------------------------------------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub filename: String,
    pub alt_text: Option<String>,
}

// ------------------------------------------------------------
// Ingredient — Python: class Ingredient
// ------------------------------------------------------------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ingredient {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub quantity: String,
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub headline: bool,
    #[serde(default)]
    pub sanitized_name: String,
}

// ------------------------------------------------------------
// Step — Python: class Step
//
// attachments 配列から image_url を取り出す処理は
// カスタムデシリアライザで行う。
// ------------------------------------------------------------
#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub description: String,
    pub id: u64,
    pub image_url: Option<String>,
}

// Step の attachments から image_url を抽出するためのヘルパー構造体
#[derive(Deserialize)]
struct RawStep {
    #[serde(default)]
    description: String,
    #[serde(default)]
    id: u64,
    #[serde(default)]
    attachments: Vec<Value>,
}

impl<'de> serde::de::Deserialize<'de> for Step {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let raw = RawStep::deserialize(d)?;

        // Python: parse_step() の attachments ループに対応
        let image_url = raw.attachments.iter().find_map(|att| {
            // パターン 1: att["url"]
            if let Some(url) = att.get("url").and_then(Value::as_str) {
                return Some(url.to_string());
            }
            // パターン 2: att["image"]["url"]
            att.get("image")
                .and_then(|img| img.get("url"))
                .and_then(Value::as_str)
                .map(String::from)
        });

        Ok(Step { description: raw.description, id: raw.id, image_url })
    }
}

// ------------------------------------------------------------
// User — Python: class User
// ------------------------------------------------------------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub profile_message: String,
    // image.url を抽出するカスタムデシリアライザ
    #[serde(default, deserialize_with = "deserialize_image_url")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub recipe_count: u64,
    #[serde(default)]
    pub follower_count: u64,
    #[serde(default)]
    pub followee_count: u64,
    #[serde(default)]
    pub cookpad_id: String,
    #[serde(default)]
    pub href: String,
}

// Python: parse_user() / parse_recipe() の img.get("url") に対応
// JSON の "image": { "url": "..." } から url を取り出す
fn deserialize_image_url<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Value> = Option::deserialize(deserializer)?;
    Ok(val.as_ref()
        .and_then(|v| v.get("url"))
        .and_then(Value::as_str)
        .map(String::from))
}

// ------------------------------------------------------------
// Recipe — Python: class Recipe
// ------------------------------------------------------------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipe {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub story: String,
    #[serde(default)]
    pub serving: String,
    pub cooking_time: Option<String>,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub hall_of_fame: bool,
    #[serde(default)]
    pub cooksnaps_count: u64,
    #[serde(default, deserialize_with = "deserialize_image_url")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    pub user: Option<User>,
    // 詳細取得時のみ設定される
    #[serde(default)]
    pub advice: String,
    #[serde(default)]
    pub bookmarks_count: u64,
    #[serde(default)]
    pub view_count: u64,
    #[serde(default)]
    pub comments_count: u64,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub href: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub premium: bool,
}

// ------------------------------------------------------------
// Comment — Python: class Comment
// ------------------------------------------------------------
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Comment {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub label: String,
    pub user: Option<User>,
    #[serde(default, deserialize_with = "deserialize_image_url")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub cursor: String,
    #[serde(default)]
    pub likes_count: u64,
    #[serde(default)]
    pub replies_count: u64,
}

// ------------------------------------------------------------
// SearchResponse — Python: class SearchResponse
//
// result 配列から type == "search_results/recipe" のみを抽出する。
// ------------------------------------------------------------
#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub recipes: Vec<Recipe>,
    pub total_count: u64,
    pub next_page: Option<u64>,
}

// Python: parse_search_response() に対応するカスタムデシリアライザ
impl<'de> serde::de::Deserialize<'de> for SearchResponse {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw: Value = Value::deserialize(d)?;

        // result 配列から type == "search_results/recipe" のみ取り出す
        let recipes: Vec<Recipe> = raw
            .get("result")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter(|item| {
                        item.get("type")
                            .and_then(Value::as_str)
                            .map(|t| t == "search_results/recipe")
                            .unwrap_or(false)
                    })
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let extra = raw.get("extra");
        let total_count = extra
            .and_then(|e| e.get("total_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let next_page = extra
            .and_then(|e| e.get("links"))
            .and_then(|l| l.get("next"))
            .and_then(|n| n.get("page"))
            .and_then(Value::as_u64);

        Ok(SearchResponse { recipes, total_count, next_page })
    }
}

// ------------------------------------------------------------
// CommentsResponse — Python: class CommentsResponse
// ------------------------------------------------------------
#[derive(Debug, Clone, Serialize)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
    pub next_cursor: Option<String>,
}

impl<'de> serde::de::Deserialize<'de> for CommentsResponse {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw: Value = Value::deserialize(d)?;

        let comments: Vec<Comment> = raw
            .get("result")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Python: comments[-1].cursor に対応
        let next_cursor = comments
            .last()
            .and_then(|c| if c.cursor.is_empty() { None } else { Some(c.cursor.clone()) });

        Ok(CommentsResponse { comments, next_cursor })
    }
}

// ------------------------------------------------------------
// UsersResponse — Python: class UsersResponse
// ------------------------------------------------------------
#[derive(Debug, Clone, Serialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
    pub total_count: u64,
    pub next_page: Option<u64>,
}

impl<'de> serde::de::Deserialize<'de> for UsersResponse {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw: Value = Value::deserialize(d)?;

        let users: Vec<User> = raw
            .get("result")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let extra = raw.get("extra");
        let total_count = extra
            .and_then(|e| e.get("total_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let next_page = extra
            .and_then(|e| e.get("links"))
            .and_then(|l| l.get("next"))
            .and_then(|n| n.get("page"))
            .and_then(Value::as_u64);

        Ok(UsersResponse { users, total_count, next_page })
    }
}
