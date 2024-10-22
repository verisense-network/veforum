use parity_scale_codec::{Decode, Encode};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, isize};
use vrs_core_sdk::{
    callback,
    http::{self, *},
    now, set_timer, timer, CallResult,
};
use vrs_core_sdk::{get, post, storage};

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}
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
            headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", constants::OPENAI_API_KEY),
            );
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            let request_body = format!(
                r#"{{
                    "model": "gpt-4",
                    "messages": [
                        {{"role": "system", "content": "You are a helpful and engaging reply bot designed to respond to comments on a forum. Your goal is to provide informative, friendly, and contextually relevant responses to each comment you receive. Please ensure your replies are polite, concise, and add value to the conversation. If a comment asks a question, try to provide a clear and accurate answer. If the comment is an opinion, acknowledge it and offer additional insights or related information. Always maintain a positive and respectful tone.\nExample:\n1. Comment: What are the benefits of using Rust for web development?\nReply: Rust offers several benefits for web development, including memory safety, high performance, and a strong type system that helps catch bugs at compile time. Additionally, frameworks like Actix and Rocket make it easier to build robust web applications in Rust.\n2. Comment: I think Python is better than Rust for beginners.\nReply: Python is indeed a great language for beginners due to its simple syntax and vast community support. However, Rust can be a good choice for those interested in systems programming and learning about memory management. Both languages have their strengths depending on your goals.\n3. Comment: Can someone explain async programming in simple terms?\nReply: Async programming allows your program to perform tasks without blocking the main thread, meaning it can handle other tasks while waiting for long operations to complete. This is especially useful in web servers, where handling multiple requests simultaneously is important.\nThis prompt sets clear expectations for the bot's behavior and provides examples to guide the generation of responses. You can customize the prompt further based on the specific needs of your application or forum."}},
                        {{"role": "user", "content": "{}"}}
                    ]
                }}"#,
                article.content
            );
            vrs_core_sdk::println!("request_body: {}", request_body);

            let request_head = RequestHead {
                method: HttpMethod::Post,
                uri: url.to_string(),
                headers,
            };

            let request = HttpRequest {
                head: request_head,
                body: request_body.into_bytes(),
            };
            let id = http::request(request).map_err(|e| e.to_string())?;
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
    let _ = storage::get(&[PREFIX_REQUEST_ARTICLE_ID_MAPPING, &id.to_be_bytes()[..]].concat())
        .map_err(|e| e.to_string())
        .and_then(|article_id| {
            if let Some(article_id) = article_id {
                let article_id =
                    u64::from_be_bytes(article_id.try_into().expect("Slice with incorrect length"));
                match response {
                    Ok(response) => {
                        let body = String::from_utf8_lossy(&response.body);
                        vrs_core_sdk::println!("id = {}, response: {}", id, body);
                        let parsed: ApiResponse =
                            serde_json::from_str(&body).map_err(|e| e.to_string())?;

                        // Extract the content from the first choice
                        if let Some(first_choice) = parsed.choices.get(0) {
                            println!("Content: {}", first_choice.message.content.clone());
                            reply_article(first_choice.message.content.clone(), article_id)
                                .map_err(|e| e.to_string())
                        } else {
                            vrs_core_sdk::eprintln!("id = {}, error: No choices available.", id);
                            return Err("No choices available.".to_string());
                        }
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
