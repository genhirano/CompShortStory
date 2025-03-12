extern crate rocket;
use std::env;
use std::path::Path;
use anyhow::Context;

use chrono::{ DateTime, Utc };
use chrono_tz::Asia::Tokyo;
use reqwest::header::HeaderMap;
use reqwest::Client;
use rocket::form::Form;
use rocket::fs::{ relative, FileServer };
use rocket::serde::json::serde_json;
use rocket::serde::Serialize;
use rocket::{ get, post, routes };
use rocket::{ FromForm, State };
use rocket_dyn_templates::Template;
use serde_json::Value;
use shuttle_runtime::SecretStore;

use rocket::serde::json::Json;
use rocket_cors::CorsOptions;

// Shuttle の secret に設定された値
struct MyState {
    secret: String,
}

// HTMLテンプレートに渡すデータを定義
#[derive(Serialize, Debug)]
struct WebContext {
    title: String,
    version: String,
    chatgpt: Vec<String>,
    claude: Vec<String>,
    gemini: Vec<String>,
    copilot: Vec<String>,
    prompt: Vec<String>,
    deepseek: Vec<String>,
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
        .unwrap_or("")
        .split('\n')
        .map(|s| {
            if s.is_empty() { "　".to_string() } else { s.to_string() }
        })
        .collect()
}

// microCMSからデータを取得
async fn getdata_from_microcms(api_key: &str, offset: i64) -> Result<WebContext, String> {
    let endpoint =
        format!("https://wa4vehv99r.microcms.io/api/v1/aitext?limit=1&orders=-date&offset={}", offset);

    // APIキーをヘッダーに設定
    let mut headers = HeaderMap::new();
    headers.insert("X-API-KEY", api_key.parse().expect("APIキーのパースに失敗しました"));
    //.parse()メソッドは、FromStrトレイトを実装している任意の型に対して使用できます。このメソッドは、Result型を返します。

    // HTTPクライアントを作成
    let client = Client::new();

    // APIでコンテンツを取得
    let response = client.get(endpoint).headers(headers).send().await;
    if response.is_err() {
        return Err("APIエラー".to_string());
    }
    let response = response.expect("APIリクエストの送信に失敗しました");

    // コンテンツをJsonに解釈
    let value = response.json::<serde_json::Value>().await;
    if value.is_err() {
        return Err("JSONエラー".to_string());
    }
    let value = value.expect("JSONのパースに失敗しました");

    //記事の総数と現在位置を取得
    let totalcount = take_value_from_json(&value, "totalCount").unwrap_or(0);
    let offset = take_value_from_json(&value, "offset").unwrap_or(0);

    //（次ボタンや前ボタンの表示制御に使用）
    let has_prev = totalcount - offset > 1;
    let has_next = offset > 0;

    //記事本文取得（日付降順ソートされた配列の０番目の一つのみ取得）
    let body_data = value.as_object();
    if body_data.is_none() {
        return Err("JSONエラー".to_string());
    }
    let contents = match body_data.expect("JSONオブジェクトの取得に失敗しました").get("contents") {
        Some(contents) => contents,
        None => return Err("コンテンツが見つかりません".to_string()),
    };
    if contents.as_array().expect("コンテンツの配列取得に失敗しました").is_empty() {
        return Err("コンテンツが見つかりません".to_string());
    }
    let value = &contents[0];

    // 記事の日付をJSTに変換（日付データはISO 8601形式のUTC（協定世界時））
    let datetime_str = match value["date"].as_str() {
        Some(date) => date.to_string(),
        None => return Err("Date not found in JSON".to_string()),
    };
    let datetime_utc: DateTime<Utc> = match datetime_str.parse() {
        Ok(datetime) => datetime,
        Err(_) => return Err("Failed to parse date".to_string()),
    };
    let datetime_jst = datetime_utc.with_timezone(&Tokyo);
    let date_str = datetime_jst.format("%Y-%m-%d").to_string();

    // レスポンスの内容をセット
    let context = WebContext {
        title: value["title"].as_str().unwrap_or("").to_string(),
        version: date_str.to_string(),
        chatgpt: take_value_from_json_with_line(&value, "ChatGPT"),
        claude: take_value_from_json_with_line(&value, "Claude"),
        gemini: take_value_from_json_with_line(&value, "Gemini"),
        copilot: take_value_from_json_with_line(&value, "Copilot"),
        prompt: take_value_from_json_with_line(&value, "prompt"),
        deepseek: take_value_from_json_with_line(&value, "deepseek"),
        totalcount: totalcount,
        offset: offset,
        has_next: has_next,
        has_prev: has_prev,
    };

    Ok(context)
}

#[get("/")]
async fn index(state: &State<MyState>) -> Template {
    //最新記事（offset=0）を取得
    let context = match getdata_from_microcms(state.secret.as_str(), 0).await {
        Ok(context) => context,
        Err(message) => return Template::render("error", serde_json::json!({"message":message})),
    };

    //ページ遷移
    Template::render("index", &context)
}

// HTTP フォームデータ
#[derive(FromForm)]
struct PostData {
    direction: String,
    currentoffset: i64,
}

#[post("/", data = "<arg>")]
async fn post_index(arg: Form<PostData>, state: &State<MyState>) -> Template {
    // 次ボタン「next」、前ボタン「prev」
    let direction = &arg.direction;

    //現在の記事位置
    let current_offset = &arg.currentoffset;

    //ページ遷移後の記事位置
    let offset = current_offset + (if direction == "next" { -1 } else { 1 });

    //記事データ取得
    let context = match getdata_from_microcms(state.secret.as_str(), offset).await {
        Ok(context) => context,
        Err(message) => return Template::render("error", serde_json::json!({"message":message})),
    };

    //ページ遷移
    Template::render("index", &context)
}

#[derive(FromForm)]
struct GetRequestParam {
    direction: String,
    currentoffset: i64,
}

#[get("/api?<query..>")]
async fn api(query: GetRequestParam, state: &State<MyState>) -> Json<WebContext> {
    //http://127.0.0.1:8000/api?direction=1&currentoffset=2&aa=d

    if !(query.direction == "next" || query.direction == "prev" || query.direction == "now") {
        return create_err_json_data("directionの値が不正です".to_string());
    }

    let offset = match query.direction.as_str() {
        "next" => query.currentoffset - 1,
        "prev" => query.currentoffset + 1,
        _ => 0, // "now"の場合とそれ以外
    };

    let context = match getdata_from_microcms(state.secret.as_str(), offset).await {
        Ok(context) => context,
        Err(message) => return create_err_json_data(message),
    };

    Json(context)
}

fn create_err_json_data(reson: String) -> Json<WebContext> {
    Json(WebContext {
        title: reson.to_string(),
        version: "エラー".to_string(),
        chatgpt: vec!["エラー".to_string()],
        claude: vec!["エラー".to_string()],
        gemini: vec!["エラー".to_string()],
        copilot: vec!["エラー".to_string()],
        prompt: vec!["エラー".to_string()],
        deepseek: vec!["エラー".to_string()],
        totalcount: 0,
        offset: 0,
        has_next: false,
        has_prev: false,
    })
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    //Shuttle の secret データ取得
    let secret = secrets.get("MICROCMS_KEY").context("secret was not found")?;
    let state = MyState { secret };

    let cors = CorsOptions::default()
        .to_cors()
        .expect("CorsOptions failed to create a CORS fairing");

    //assetsの絶対パスを取得
    let asstspath = Path::new(relative!("assets"));

    let rocket = rocket
        ::build()
        .mount("/", routes![index, post_index, api])
        .mount("/", FileServer::from(asstspath)) // 静的ファイルのPath設定。デフォルトは staticだが、assetsに変更する
        .manage(state)
        .attach(Template::fairing())
        .attach(cors);

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

    const TESTDATA_ONE: &'static str =
        r#"
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
