use chrono::{DateTime, Utc};
use chrono_tz::Asia::Tokyo;
use reqwest::header::HeaderMap;
use reqwest::Client;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::serde::json::serde_json;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::{get, post, routes};
use rocket::{FromForm, State};
use rocket_dyn_templates::Template;

struct MyState {
    secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonContent {
    id: String,
    created_at: String,
    updated_at: String,
    published_at: String,
    revised_at: String,
    date: String,
    title: String,
    prompt: String,
    chat_gpt: String,
    claude: String,
    gemini: String,
    copilot: String,
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
                        let totalcount = obj.get("totalCount");
                        let totalcount = match totalcount {
                            Some(totalcount) => totalcount.as_i64().unwrap(),
                            None => {
                                panic!("totalcountが取得できませんでした");
                            }
                        };
                        let offset = obj.get("offset");
                        let offset = match offset {
                            Some(offset) => offset.as_i64().unwrap(),
                            None => {
                                panic!("offsetが取得できませんでした");
                            }
                        };

                        let has_prev = (totalcount - offset) > 1;
                        let has_next = offset > 0;

                        let root_node = obj.get("contents");
                        if let Some(root_node) = root_node {
                            if let Some(contents) = root_node.as_array() {
                                for content in contents {
                                    if let Some(obj) = content.as_object() {
                                        
                                        // 取得した日付をJSTに変換（日付データはISO 8601形式のUTC（協定世界時））
                                        let datetime_str =
                                            obj["date"].as_str().unwrap().to_string();
                                        let datetime_utc: DateTime<Utc> =
                                            datetime_str.parse().unwrap();
                                        let datetime_jst = datetime_utc.with_timezone(&Tokyo);
                                        println!("datetime_jst:{}", datetime_jst);
                                        let date_str = datetime_jst.format("%Y-%m-%d").to_string();

                                        context = Some(WebContext {
                                            title: obj["title"].as_str().unwrap().to_string(),
                                            version: date_str.to_string(),
                                            chatgpt: obj["ChatGPT"]
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
                                                .collect(),
                                            claude: obj["Claude"]
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
                                                .collect(),
                                            gemini: obj["Gemini"]
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
                                                .collect(),
                                            copilot: obj["Copilot"]
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
                                                .collect(),
                                            prompt: obj["prompt"]
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
                                                .collect(),
                                            totalcount: totalcount,
                                            offset: offset,
                                            has_next: has_next,
                                            has_prev: has_prev,
                                        });
                                    }
                                }
                            }
                        }
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

    println!("currentoffset:{}", currentoffset);
    println!("direction:{}", direction);

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
