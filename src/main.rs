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

struct MyState {
    secret: String,
}

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

async fn getdata_from_microcms(api_key: &str, offset: i64) -> Option<WebContext> {
    // microCMSのAPIエンドポイントとAPIキー
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

    let mut context: Option<WebContext> = None;

    match response {
        Ok(resp) => {
            // レスポンスのボディをJSONとしてパースして返す
            match resp.json::<serde_json::Value>().await {
                Ok(content) => {
                    if let Some(obj) = content.as_object() {
                        
                        //登録済みの全件数とオフセットを取得
                        let totalcount = obj.get("totalCount").unwrap().as_i64().unwrap();
                        let offset = obj.get("offset").unwrap().as_i64().unwrap();

                        let has_prev = (totalcount - offset) > 1;
                        let has_next = offset > 0;

                        let value = &obj["contents"][0];

                        // 取得した日付をJSTに変換（日付データはISO 8601形式のUTC（協定世界時））
                        let datetime_str = &value["date"].as_str().unwrap().to_string();
                        let datetime_utc: DateTime<Utc> = datetime_str.parse().unwrap();
                        let datetime_jst = datetime_utc.with_timezone(&Tokyo);
                        let date_str = datetime_jst.format("%Y-%m-%d").to_string();

                        // レスポンスの内容をセット
                        context = Some(WebContext {
                            title: value["title"].as_str().unwrap().to_string(),
                            version: date_str.to_string(),
                            chatgpt: get_string_from_value(&value, "ChatGPT"),
                            claude: get_string_from_value(&value, "Claude"),
                            gemini: get_string_from_value(&value, "Gemini"),
                            copilot: get_string_from_value(&value, "Copilot"),
                            prompt: get_string_from_value(&value, "prompt"),
                            totalcount: totalcount,
                            offset: offset,
                            has_next: has_next,
                            has_prev: has_prev,
                        });
                    }
                }
                Err(_) => panic!("パニック1"),
            }
        }
        Err(_) => panic!("パニック2"),
    }

    return context;
}

#[get("/")]
async fn index(state: &State<MyState>) -> Template {
    let context = getdata_from_microcms(state.secret.as_str(), 0);
    match context.await {
        Some(context) => Template::render("index", &context),
        None => panic!("パニック3"),
    }
}

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
        Some(context) => Template::render("index", &context),
        None => panic!("パニック3"),
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    // memo
    // cargo shuttle resource delete secrets  ※Deploy済みのシークレットをShuttleから全部削除

    //let secret = secrets.get("secret_key").context("secret was not found")?;
    let secret = "pfcBd8oLiiBW240e7I9IDjy6jWXHkaLE2Qx2".to_string();

    let state = MyState { secret };

    let rocket = rocket::build()
        .mount("/", routes![index, post_index])
        .mount("/", FileServer::from(relative!("static")))
        .manage(state)
        .attach(Template::fairing());

    Ok(rocket.into())
}

fn get_string_from_value(data: &Value, key: &str) -> Vec<String> {
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

//test
#[cfg(test)]
mod tests {
    use super::*;

    const HAIKU: &'static str = r#"
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
        let obj: Result<Value, serde_json::Error> = serde_json::from_str(HAIKU);
        print!("{:?}", obj.unwrap());
    }
}
