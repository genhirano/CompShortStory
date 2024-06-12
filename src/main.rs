extern crate rocket;
use anyhow::Context;

use chrono::{DateTime, Utc};
use chrono_tz::Asia::Tokyo;
use reqwest::header::HeaderMap;
use reqwest::Client;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::serde::json::serde_json;
use rocket::serde::Serialize;
use rocket::{get, post, routes};
use rocket::{FromForm, State};
use rocket_dyn_templates::Template;
use serde_json::Value;
use shuttle_runtime::SecretStore;

// Shuttle の secret に設定された値
struct MyState {
    secret: String,
}

// テンプレートに渡すデータを定義
#[derive(Serialize)]
struct WebContext {
    title: String,
    version: String,
    chatgpt: Vec<String>,
    claude: Vec<String>,
    gemini: Vec<String>,
    copilot: Vec<String>,
    prompt: Vec<String>,
    totalcount: i64,
    offset: i64,
    has_next: bool,
    has_prev: bool,
}

// Json (serde_json::value) から文字列を取得してそのまま返す
fn take_value_from_json(obj: &Value, key: &str) -> Result<i64, String> {
    let text = obj
        .get(&key)
        .ok_or(key.to_string() + " not found")?
        .as_i64()
        .ok_or(key.to_string() + " is not an integer")?;

    Ok(text)
}

// Json (serde_json::value) から、改行コードを含む文字列を取得して、各行のVecterにして返す
fn take_value_from_json_with_line(data: &Value, key: &str) -> Vec<String> {
    data[key]
        .as_str()
        .unwrap()
        .split('\n')
        .map(|s| {
            if s.is_empty() {
                "　".to_string()
            } else {
                s.to_string()
            }
        })
        .collect()
}

// microCMSからデータを取得
async fn getdata_from_microcms(api_key: &str, offset: i64) -> Result<WebContext, String> {
    let endpoint = format!(
        "https://wa4vehv99r.microcms.io/api/v1/aitext?limit=1&orders=-date&offset={}",
        offset
    );

    // APIキーをヘッダーに設定
    let mut headers = HeaderMap::new();
    headers.insert("X-API-KEY", api_key.parse().unwrap());

    // HTTPクライアントを初期化
    let client = Client::new();

    // コンテンツを取得
    let response = client.get(endpoint).headers(headers).send().await;
    if response.is_err() {
        return Err("APIエラー".to_string());
    }
    let response = response.unwrap();

    let value = response.json::<serde_json::Value>().await;
    if value.is_err() {
        return Err("JSONエラー".to_string());
    }
    let value = value.unwrap();

    let totalcount = take_value_from_json(&value, "totalCount").unwrap();
    let offset = take_value_from_json(&value, "offset").unwrap();

    //登録記事の現在位置を把握（次ボタンや前ボタンの表示制御に使用）
    let has_prev = (totalcount - offset) > 1;
    let has_next = offset > 0;

    //記事取得（日付降順ソートされた配列の０番目の一つのみ取得）
    let body_data = value.as_object();
    if body_data.is_none() {
        return Err("JSONエラー".to_string());
    }
    let value = &body_data.unwrap()["contents"][0];

    // 記事の日付をJSTに変換（日付データはISO 8601形式のUTC（協定世界時））
    let datetime_str = &value["date"].as_str().unwrap().to_string();
    let datetime_utc: DateTime<Utc> = datetime_str.parse().unwrap();
    let datetime_jst = datetime_utc.with_timezone(&Tokyo);
    let date_str = datetime_jst.format("%Y-%m-%d").to_string();

    // レスポンスの内容をセット
    let context = WebContext {
        title: value["title"].as_str().unwrap().to_string(),
        version: date_str.to_string(),
        chatgpt: take_value_from_json_with_line(&value, "ChatGPT"),
        claude: take_value_from_json_with_line(&value, "Claude"),
        gemini: take_value_from_json_with_line(&value, "Gemini"),
        copilot: take_value_from_json_with_line(&value, "Copilot"),
        prompt: take_value_from_json_with_line(&value, "prompt"),
        totalcount: totalcount,
        offset: offset,
        has_next: has_next,
        has_prev: has_prev,
    };

    Ok(context)
}

#[get("/")]
async fn index(state: &State<MyState>) -> Template {
    let context = getdata_from_microcms(state.secret.as_str(), 0);
    match context.await {
        Ok(context) => Template::render("index", &context),
        Err(message) => Template::render("error", serde_json::json!({"message":message})),
    }
}

// HTTP フォームデータ
#[derive(FromForm)]
struct PostData {
    direction: String,
    currentoffset: i64,
}

#[post("/", data = "<arg>")]
async fn post_index(arg: Form<PostData>, state: &State<MyState>) -> Template {
    let direction = &arg.direction;
    let currentoffset = &arg.currentoffset;

    let offset = currentoffset + if direction == "next" { -1 } else { 1 };

    let context = getdata_from_microcms(state.secret.as_str(), offset);
    match context.await {
        Ok(context) => Template::render("index", &context),
        Err(message) => Template::render("error", serde_json::json!({"message":message})),
    }
}
#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let secret = secrets
        .get("MICROCMS_KEY")
        .context("secret was not found")?;

    let state = MyState { secret };

    let rocket = rocket::build()
        .mount("/", routes![index, post_index])
        .mount("/", FileServer::from(relative!("assets"))) // 静的ファイルのPath設定。デフォルトは staticだが、assetsに変更する
        .manage(state)
        .attach(Template::fairing());

    Ok(rocket.into())
}

//test
#[cfg(test)]
mod tests {
    use super::*;

    /*
    定数について
    ライフサイクル指定する 'staticは、プログラムが終了するまでメモリ上に存在する
    実際には const は 'static と同じライフタイムを持つが、'static は明示的に指定することで、プログラムの終了までメモリ上に存在することを示す
    （&str でも良いか 'static を書いた方が明示的でわかりやすい）

     */

    const TESTDATA_ONE: &'static str = r#"
    {
        "contents": [
            {
                "id": "6c2h_8djem9b",
                "createdAt": "2024-06-04T07:47:44.916Z",
                "updatedAt": "2024-06-04T07:47:44.916Z",
                "publishedAt": "2024-06-04T07:47:44.916Z",
                "revisedAt": "2024-06-04T07:47:44.916Z",
                "date": "2024-05-24T15:00:00.000Z",
                "title": "”行けたら行く”と言っていたのに本当に来たやつ",
                "prompt": "あなたは小説家です。小説を書いてください。\n・タイトル：「”行けたら行く”と言っていたのに本当に来たやつ」\n",
                "ChatGPT": "桜井はいつも遅刻しがちで、誘われても「行けたら行く」と曖昧な返事をする男だった。",
                "Claude": "第一段落:\n俺は謎の男から不思議な伝言を受け取った。",
                "Gemini": "薄暗い薄暗い森の奥深く、一軒の古びた小屋があった。",
                "Copilot": "ある晴れた日、小さな村の広場で、不思議な出来事が起こりました。"
            }
        ],
        "totalCount": 13,
        "offset": 0,
        "limit": 1
    }
    "#;

    #[test]
    fn test_json() {
        let obj: Result<Value, serde_json::Error> = serde_json::from_str(TESTDATA_ONE);
        print!("{:?}", obj.unwrap());
    }
}
