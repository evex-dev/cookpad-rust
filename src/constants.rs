// Python の constants.py に対応する定数定義。

pub const BASE_URL: &str = "https://global-api.cookpad.com/v32";
pub const API_HOST: &str = "global-api.cookpad.com";

pub const DEFAULT_TOKEN: &str =
    "54ccbf3be26f7d3d3c1e068d53032e98e3ff992d49979f8e120b323910f0b942";

pub const DEFAULT_USER_AGENT: &str =
    "com.cookpad/2026.7.0; iOS/26.2.1; iPhone17,3; ; ja_JP;";

pub const DEFAULT_COUNTRY: &str = "JP";
pub const DEFAULT_LANGUAGE: &str = "ja";
pub const DEFAULT_TIMEZONE_ID: &str = "Asia/Tokyo";
pub const DEFAULT_TIMEZONE_OFFSET: &str = "+09:00";
pub const DEFAULT_PROVIDER_ID: &str = "8";

pub const SUPPORTED_SEARCH_TYPES: &str = concat!(
    "search_results/recipe",
    ",search_results/visual_guides",
    ",search_results/spelling_suggestion",
    ",search_results/title",
    ",search_results/add_recipe_prompt",
    ",search_results/premium_recipe_carousel",
    ",search_results/premium_recipe_promotion",
    ",search_results/library_recipes",
    ",search_results/delicious_ways",
    ",search_results/popular_promo_recipe",
    ",search_results/premium_banner",
);
