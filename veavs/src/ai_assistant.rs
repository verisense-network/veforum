use parity_scale_codec::{Decode, Encode};
use std::{collections::BTreeMap, isize};
use vrs_core_sdk::{
    callback,
    http::{self, *},
    now, set_timer, timer, CallResult,
};
use vrs_core_sdk::{get, post, storage};

use vemodel::{
    Method, VeArticle, VeComment, VeSubspace, COMMON_KEY, PREFIX_ARTICLE_KEY, PREFIX_COMMENT_KEY,
    PREFIX_SUBSPACE_KEY, REQNUM_KEY,
};
const PREFIX_ARTICLE_PROCESSING_KEY: &[u8; 5] = b"rear:";
const PREFIX_REQUEST_ARTICLE_ID_MAPPING: &[u8; 5] = b"reqm:";
use super::*;

#[get]
fn fetch_lastest_article() -> Result<Vec<VeArticle>, String> {
    let mut articles = vec![];
    let max_id_key = [PREFIX_ARTICLE_KEY, &u64::MAX.to_be_bytes()[..]].concat();
    match storage::get_range(PREFIX_ARTICLE_KEY, storage::Direction::Forward, 100)
        .map_err(|e| e.to_string())
    {
        Ok(vec) => {
            for v in vec {
                if v.0.starts_with(PREFIX_ARTICLE_KEY) {
                    let article = VeArticle::decode(&mut &v.1[..]).map_err(|e| e.to_string())?;
                    articles.push(article);
                }
                // articles.push(v.0);
            }
        }
        Err(e) => {
            return Err(e);
        }
    };
    Ok(articles)
}
#[get]
fn check_article_processing(article_id: u64) -> Result<bool, String> {
    let key = [PREFIX_ARTICLE_PROCESSING_KEY, &article_id.to_be_bytes()[..]].concat();
    match storage::get(&key).map_err(|e| e.to_string())? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

fn reply_article(content: String, article_id: u64) -> Result<u64, String> {
    let max_id = get_max_id(PREFIX_COMMENT_KEY);
    let comment = VeComment {
        id: max_id,
        content: content.clone(),
        author_id: 0,
        author_nickname: "AI Assistant".to_string(),
        article_id: article_id,
        status: 0,
        weight: 0,
        created_time: now() as i64,
    };
    let key = build_key(PREFIX_COMMENT_KEY, max_id);
    storage::put(&key, comment.encode()).map_err(|e| e.to_string())?;
    add_to_common_key(Method::Create, key)?;

    let key = [PREFIX_ARTICLE_PROCESSING_KEY, &article_id.to_be_bytes()[..]].concat();
    storage::put(&key, comment.encode()).map_err(|e| e.to_string())?;
    vrs_core_sdk::println!("{:?}", comment);

    Ok(max_id)
}
#[post]
fn reply_all_articles() -> Result<(), String> {
    let articles = fetch_lastest_article()?;
    for article in articles {
        if !check_article_processing(article.id)? {
            let url = "https://api.openai.com/v1/chat/completions";

            let mut headers = BTreeMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            let request_body = json!({
                "model": "gpt-3.5-turbo",
                "messages": [
                    {"role": "user", "content": post_content}
                ]
            });

            let request_head = RequestHead {
                method: HttpMethod::Post,
                uri: url.to_string(),
                headers,
            };

            let request = HttpRequest {
                head: request_head,
                body: serde_json::to_vec(&request_body)?,
            };

            let id = http::request(HttpRequest {
                head: RequestHead {
                    method: HttpMethod::Get,
                    uri: "https://www.baidu.com".to_string(),
                    headers: Default::default(),
                },
                body: vec![],
            })
            .map_err(|e| e.to_string())?;
            vrs_core_sdk::println!("http request {} enqueued", id);

            let key = [PREFIX_ARTICLE_PROCESSING_KEY, &article.id.to_be_bytes()[..]].concat();
            storage::put(&key, &id.to_be_bytes()).map_err(|e| e.to_string())?;

            let key = [PREFIX_REQUEST_ARTICLE_ID_MAPPING, &id.to_be_bytes()[..]].concat();
            storage::put(&key, &article.id.to_be_bytes()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[timer]
fn timer_reply_all_articles() {
    reply_all_articles();
    set_timer!(5, timer_reply_all_articles);
}

#[callback]
pub fn on_response(id: u64, response: CallResult<HttpResponse>) {
    storage::get(&[PREFIX_REQUEST_ARTICLE_ID_MAPPING, &id.to_be_bytes()[..]].concat())
        .map_err(|e| e.to_string())
        .and_then(|article_id| {
            if let Some(article_id) = article_id {
                let article_id =
                    u64::from_be_bytes(article_id.try_into().expect("Slice with incorrect length"));
                match response {
                    Ok(response) => {
                        let body = String::from_utf8_lossy(&response.body);
                        // vrs_core_sdk::println!("id = {}, response: {}", id, body);
                        reply_article(body.to_string(), article_id).map_err(|e| e.to_string())
                    }
                    Err(e) => {
                        vrs_core_sdk::eprintln!("id = {}, error: {:?}", id, e);
                        Err(e.to_string())
                    }
                }
            } else {
                return Err("article_id not found".to_string());
            }
        })
        .map_err(|e| vrs_core_sdk::eprintln!("error: {:?}", e));
}
