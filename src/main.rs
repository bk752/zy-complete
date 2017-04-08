extern crate hyper;
extern crate hyper_native_tls;
extern crate rustc_serialize;
extern crate rpassword;
extern crate flate2;
extern crate time;
#[macro_use]
extern crate language_tags;

use std::io;
use std::io::Read;
use hyper::Client;
use hyper::header::*;
use hyper::mime::*;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use rpassword::read_password;
use rustc_serialize::json::Json;
use flate2::read::GzDecoder;

// TODO: Handle incorrect input, unexpected server responses, errors, etc

/// Get user ID and token from login information
fn login_user(user: String, pass: String, client: &Client, head: Headers) -> (u64, String) {
    // create body
    let info = format!(r#"{{"email":"{}","password":"{}"}}"#, user, pass);

    // make request and get response
    let mut res = client.post("https://s1.zybooks.com/v1/signin")
        .body(&*info)
        .headers(head)
        .send()
        .unwrap();

    let mut data = String::new();
    res.read_to_string(&mut data).unwrap();


    let login = Json::from_str(&data).unwrap();
    let user_id = login.search("user_id").unwrap().as_u64().unwrap();
    let token = login.search("auth_token").unwrap().as_string().unwrap().to_owned();

    (user_id, token)
}

/// Get list of books owned by user
fn get_books(user_id: u64, token: &str, client: &Client, head: Headers) -> Vec<String> {

    let url = format!(r#"https://s1.zybooks.com/v1/user/{}/zybooks?auth_token={}"#, user_id, token);

    let mut res = client.get(&url)
        .headers(head)
        .send()
        .unwrap();

    // decode response from server
    let mut raw: Vec<u8> = Vec::new();
    res.read_to_end(&mut raw).unwrap();
    let mut gz = GzDecoder::new(raw.as_slice()).unwrap();

    let mut data = String::new();
    gz.read_to_string(&mut data).unwrap();

    let index = Json::from_str(&data).unwrap();
    let book_list = index.search("zybooks").unwrap().as_array().unwrap();

    let mut books: Vec<String> = Vec::new();

    // get list of books in json
    for book in book_list {
        books.push(book.search("zybook_code").unwrap().as_string().unwrap().to_owned());
    }

    books
}

/// Get list of questions in book
fn get_questions(user_id: u64, token: &str, book_code: &str, client: &Client, head: Headers) -> Vec<(String, usize)> {

    let url = format!(r#"https://s1.zybooks.com/v1/zybook/{}/activities/{}?auth_token={}"#, book_code, user_id, token);

    let mut res = client.get(&url)
        .headers(head)
        .send()
        .unwrap();

    // decode response from server
    let mut raw: Vec<u8> = Vec::new();
    res.read_to_end(&mut raw).unwrap();
    let mut gz = GzDecoder::new(raw.as_slice()).unwrap();

    let mut data = String::new();
    gz.read_to_string(&mut data).unwrap();

    let info = Json::from_str(&data).unwrap();
    let question_list = info.search("data").unwrap().as_array().unwrap()[0].as_array().unwrap();

    // find questions in response
    let mut questions: Vec<(String, usize)> =  Vec::new();
    for chapter in question_list {
        let id_list = chapter.as_object().unwrap();
        for (id, parts) in id_list {
            questions.push((id.to_owned(), parts.as_array().unwrap().len()));
        }
    }
    questions
}

/// Complete a certain part of a question
fn complete_question(token: &str, book_code: &str, id: &str, part: usize, client: &Client, head: Headers) {
    // get url of zybook answer site
    let url = format!(r#"https://s1.zybooks.com/v1/content_resource/{}/activity"#, id);

    // format timestamp
    let time = time::now_utc();
    let timestamp = format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:0>3.3}z",
                            time.tm_year, time.tm_mon, time.tm_mday,
                            time.tm_hour, time.tm_min, time.tm_sec,
                            time.tm_nsec.to_string());

    // TODO: Maybe put this in struct to make it easier to read
    let info = format!(r#"{{"part":{},"complete":true,"metadata":"","zybook_code":"{}","auth_token":"{}","timestamp":"{}"}}"#,
                       part, book_code, token, timestamp);

    // make request and get response
    let mut res = client.post(&url)
        .body(&*info)
        .headers(head)
        .send()
        .unwrap();

    let mut data = String::new();
    res.read_to_string(&mut data).unwrap();
}

/// Complete all questions in a zybook
fn main() {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    // get user info
    let mut user = String::new();
    println!("Username:");
    io::stdin().read_line(&mut user).unwrap();
    user = user.trim().to_owned();
    println!("Password:");
    let pass = read_password().unwrap();

    // create post header
    let mut head_post = Headers::new();
    head_post.set(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![])),
                         qitem(Mime(TopLevel::Text, SubLevel::Javascript, vec![])),
                         QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(10))]));
    head_post.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                 qitem(Encoding::Deflate),
                                 qitem(Encoding::EncodingExt("br".to_owned())),]));
    head_post.set(AcceptLanguage(vec![qitem(langtag!(en;;;US)),
                                 QualityItem::new(langtag!(en), Quality(800)),]));
    head_post.set(Connection::keep_alive());
    //head.set(ContentLength(34u64));
    head_post.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    head_post.set(Host{hostname: "s1.zybooks.com".to_owned(), port: None,});
    head_post.set(Origin::new("https", "zybooks.zyante.com", None));
    head_post.set(Referer("https://zybooks.zyante.com/".to_owned()));
    head_post.set(UserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/57.0.2987.133 Safari/537.36".to_owned()));

    // TODO: figure out way of calling without clone
    let (user_id, token) = login_user(user, pass, &client, head_post.clone());

    // modify header for get requests
    let mut head_get = head_post.clone();
    head_get.remove::<ContentType>();
    head_get.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                 qitem(Encoding::Deflate),
                                 qitem(Encoding::EncodingExt("sdch".to_owned())),
                                 qitem(Encoding::EncodingExt("br".to_owned())),]));

    // let user choose which book to complete
    let books = get_books(user_id, &token, &client, head_get.clone());
    for (i, book) in books.iter().enumerate() {
        println!("{}) {}", i+1, book);
    }
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice_val = choice.trim().parse::<usize>().unwrap();

    let questions = get_questions(user_id, &token, &books[choice_val-1], &client, head_get.clone());

    // complete all questions in book
    for (id, parts) in questions {
        for part in 0..parts {
            complete_question(&token, &books[choice_val-1], &id, part, &client, head_post.clone());
        }
    }
}
