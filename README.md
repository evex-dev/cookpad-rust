# cookpad-rs

Cookpad の非公式 Rust クライアント。
cookpad-py を Rust にリライトした。

## インストール

`Cargo.toml` に追加:

```toml
[dependencies]
cookpad = { git = "https://github.com/evex-dev/cookpad-rs" }
```

## 使い方

一部の引数とかは、認証済みの token じゃないと動かないので注意

```rust
use cookpad::Cookpad;

#[tokio::main]
async fn main() -> Result<(), cookpad::CookpadError> {
    let client = Cookpad::new();

    // レシピ検索
    let results = client.search_recipes("カレー").await?;
    for recipe in &results.recipes {
        println!("{} (つくれぽ: {})", recipe.title, recipe.cooksnaps_count);
    }

    // レシピ詳細
    let recipe = client.get_recipe(25410768).await?;
    println!("{}", recipe.title);
    for step in &recipe.steps {
        println!("  - {}", step.description);
    }

    Ok(())
}
```

## API

### `Cookpad::new()`

クライアント作成。デフォルトで anonymous token 使うからそのまま動く。

```rust
// デフォルト (日本語・匿名)
let client = Cookpad::new();

// カスタム
let client = Cookpad::with_config(
    "your_token",
    "JP",
    "ja",
    "Asia/Tokyo",
    "+09:00",
    "custom/1.0",
    "8",
);
```

### `search_recipes(query)` / `search_recipes_with_options(query, page, per_page)`

レシピ検索。`SearchResponse` を返す。

```rust
let results = client.search_recipes("鶏むね肉").await?;
println!("全 {} 件", results.total_count);
for recipe in &results.recipes {
    println!("  {}", recipe.title);
}

// 次のページ
if let Some(next) = results.next_page {
    let page2 = client.search_recipes_with_options("鶏むね肉", next as u32, 30).await?;
}
```

### `get_recipe(recipe_id)`

レシピ詳細を取得。`Recipe` を返す。

```rust
let recipe = client.get_recipe(25410768).await?;
println!("{}", recipe.title);
println!("{}", recipe.story);
println!("材料 ({}):", recipe.serving);
for ing in &recipe.ingredients {
    println!("  {}: {}", ing.name, ing.quantity);
}
for step in &recipe.steps {
    println!("  {}", step.description);
}
```

### `get_similar_recipes(recipe_id, page, per_page)`

似てるレシピ一覧。

```rust
let similar = client.get_similar_recipes(25410768, 1, 30).await?;
for recipe in &similar {
    println!("{}", recipe.title);
}
```

### `get_comments(recipe_id, limit, after, label)`

つくれぽ・コメント取得。

```rust
let comments = client.get_comments(18510866, 10, "", "cooksnap").await?;
for comment in &comments.comments {
    if let Some(user) = &comment.user {
        println!("{}: {}", user.name, comment.body);
    }
}

// ページネーション (カーソルベース)
if let Some(cursor) = &comments.next_cursor {
    let more = client.get_comments(18510866, 10, cursor, "cooksnap").await?;
}
```

### `search_users(query, page, per_page)`

ユーザー検索。

```rust
let users = client.search_users("test", 1, 20).await?;
for user in &users.users {
    println!("{} (レシピ数: {})", user.name, user.recipe_count);
}
```

### `search_keywords(query)`

検索サジェスト。

```rust
let suggestions = client.search_keywords("カレ").await?;
```

### `get_search_history(local_history)`

検索履歴・トレンドキーワード。

```rust
let history = client.get_search_history(&[]).await?;
```

## 型

レスポンスは全部 struct でパース済み。`serde::Serialize` も実装してるので JSON に戻せる。

- `Recipe` - レシピ (id, title, story, serving, ingredients, steps, ...)
- `Ingredient` - 材料 (name, quantity)
- `Step` - 手順 (description, image_url)
- `User` - ユーザー (id, name, recipe_count, ...)
- `Comment` - コメント/つくれぽ (body, user, image_url, ...)
- `Image` - 画像 (url, filename, alt_text)
- `SearchResponse` - 検索結果 (recipes, total_count, next_page)
- `CommentsResponse` - コメント一覧 (comments, next_cursor)
- `UsersResponse` - ユーザー一覧 (users, total_count, next_page)

## エラー

```rust
use cookpad::CookpadError;

match client.get_recipe(99999999).await {
    Ok(recipe) => println!("{}", recipe.title),
    Err(CookpadError::NotFoundError(_)) => println!("レシピが見つからない"),
    Err(CookpadError::RateLimitError)   => println!("レート制限"),
    Err(e) => println!("なんかエラー: {}", e),
}
```

## ライセンス

The Unlicense (パブリックドメイン)
