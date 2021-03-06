use std::fs::File;
use std::io::prelude::*;

use curl::easy::Easy;

use hurl::http::libcurl;
use hurl::http::libcurl::client::ClientOptions;
use hurl::http::libcurl::core::*;
use server::Server;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

pub mod server;

pub fn new_header(name: &str, value: &str) -> Header {
    Header {
        name: name.to_string(),
        value: value.to_string(),
    }
}


#[test]
fn get_easy() {
    let s = Server::new();
    s.receive(
        "\
         GET /hello HTTP/1.1\r\n\
         Host: 127.0.0.1:$PORT\r\n\
         Accept: */*\r\n\
         \r\n",
    );
    s.send("HTTP/1.1 200 OK\r\n\r\nHello World!");

    let mut data = Vec::new();
    let mut handle = Easy::new();

    handle.url(&s.url("/hello")).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    assert_eq!(data, b"Hello World!");
}


fn default_client() -> libcurl::client::Client {
    let options = ClientOptions {
        follow_location: false,
        max_redirect: None,
        cookie_file: None,
        cookie_jar: None,
        proxy: None,
        verbose: false,
    };
    libcurl::client::Client::init(options)
}

fn default_get_request(url: String) -> Request {
    Request {
        method: Method::Get,
        url,
        headers: vec![],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    }
}


// region basic

#[test]
fn test_hello() {
    let mut client = default_client();
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.version, Version::Http10);
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"Hello World!".to_vec());

    assert_eq!(response.headers.len(), 4);
    assert!(response.headers.contains(&Header { name: "Content-Length".to_string(), value: "12".to_string() }));
    assert!(response.headers.contains(&Header { name: "Content-Type".to_string(), value: "text/html; charset=utf-8".to_string() }));
    assert_eq!(response.get_header_values("Date".to_string()).len(), 1);
}

// endregion

// region http method

#[test]
fn test_put() {
    let mut client = default_client();
    let request = Request {
        method: Method::Put,
        url: "http://localhost:8000/put".to_string(),
        headers: vec![],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

#[test]
fn test_patch() {

    let mut client = default_client();
    let request = Request {
        method: Method::Patch,
        url: "http://localhost:8000/patch/file.txt".to_string(),
        headers: vec![
            Header { name: "Host".to_string(), value: "www.example.com".to_string() },
            Header { name: "Content-Type".to_string(), value: "application/example".to_string() },
            Header { name: "If-Match".to_string(), value: "\"e0023aa4e\"".to_string() },
        ],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 204);
    assert!(response.body.is_empty());

}

// endregion

// region headers

#[test]
fn test_custom_headers() {
    let mut client = default_client();
    let request = Request {
        method: Method::Get,
        url: "http://localhost:8000/custom-headers".to_string(),
        headers: vec![
            new_header("Fruit", "Raspberry"),
            new_header("Fruit", "Apple"),
            new_header("Fruit", "Banana"),
            new_header("Fruit", "Grape"),
            new_header("Color", "Green"),
        ],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

// endregion

// region querystrings

#[test]
fn test_querystring_params() {
    let mut client = default_client();
    let request = Request {
        method: Method::Get,
        url: "http://localhost:8000/querystring-params".to_string(),
        headers: vec![],
        querystring: vec![
            Param { name: "param1".to_string(), value: "value1".to_string() },
            Param { name: "param2".to_string(), value: "".to_string() },
            Param { name: "param3".to_string(), value: "a=b".to_string() },
            Param { name: "param4".to_string(), value: "1,2,3".to_string() }
        ],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

// endregion

// region form params

#[test]
fn test_form_params() {
    let mut client = default_client();
    let request = Request {
        method: Method::Post,
        url: "http://localhost:8000/form-params".to_string(),
        headers: vec![],
        querystring: vec![],
        form: vec![
            Param { name: "param1".to_string(), value: "value1".to_string() },
            Param { name: "param2".to_string(), value: "".to_string() },
            Param { name: "param3".to_string(), value: "a=b".to_string() },
            Param { name: "param4".to_string(), value: "a%3db".to_string() }
        ],
        multipart: vec![],
        cookies: vec![],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());


    // make sure you can reuse client for other request
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"Hello World!".to_vec());

}

// endregion

// region redirect

#[test]
fn test_follow_location() {
    let request = default_get_request("http://localhost:8000/redirect".to_string());

    let mut client = default_client();
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 302);
    assert_eq!(response.get_header_values("Location".to_string()).get(0).unwrap(),
               "http://localhost:8000/redirected");
    assert_eq!(client.redirect_count, 0);

    let options = ClientOptions {
        follow_location: true,
        max_redirect: None,
        cookie_file: None,
        cookie_jar: None,
        proxy: None,
        verbose: false,
    };
    let mut client = libcurl::client::Client::init(options);
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.get_header_values("Content-Length".to_string()).get(0).unwrap(), "0");
    assert_eq!(client.redirect_count, 1);


    // make sure that the redirect count is reset to 0
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"Hello World!".to_vec());
    assert_eq!(client.redirect_count, 0);
}


#[test]
fn test_max_redirect() {
    let options = ClientOptions {
        follow_location: true,
        max_redirect: Some(10),
        cookie_file: None,
        cookie_jar: None,
        proxy: None,
        verbose: false,
    };
    let mut client = libcurl::client::Client::init(options);
    let request = default_get_request("http://localhost:8000/redirect".to_string());
    let response = client.execute(&request, 5).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(client.redirect_count, 6);

    let error = client.execute(&request, 11).err().unwrap();
    assert_eq!(error, HttpError::TooManyRedirect);

}

// endregion

// region multipart

#[test]
fn test_multipart_form_data() {

    let mut client = default_client();
    let request = Request {
        method: Method::Post,
        url: "http://localhost:8000/multipart-form-data".to_string(),
        headers: vec![],
        querystring: vec![],
        form: vec![],
        multipart: vec![
            MultipartParam::Param(Param{
                name: "key1".to_string(),
                value: "value1".to_string()
            }),
            MultipartParam::FileParam(FileParam{
                name: "upload1".to_string(),
                filename: "hello.txt".to_string(),
                data: b"Hello World!".to_vec(),
                content_type: "text/plain".to_string()
            }),
            MultipartParam::FileParam(FileParam{
                name: "upload2".to_string(),
                filename: "hello.html".to_string(),
                data: b"Hello <b>World</b>!".to_vec(),
                content_type: "text/html".to_string()
            }),
            MultipartParam::FileParam(FileParam{
                name: "upload3".to_string(),
                filename: "hello.txt".to_string(),
                data: b"Hello World!".to_vec(),
                content_type: "text/html".to_string()
            }),
        ],
        cookies: vec![],
        body: vec![]

    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

// endregion

// region http body

#[test]
fn test_post_bytes() {

    let mut client = default_client();
    let request = Request {
        method: Method::Post,
        url: "http://localhost:8000/post-base64".to_string(),
        headers: vec![],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![],
        body: b"Hello World!".to_vec(),
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

// endregion

// region error

#[test]
fn test_error_could_not_resolve_host() {
    let mut client = default_client();
    let request = default_get_request("http://unknown".to_string());
    let error =  client.execute(&request, 0).err().unwrap();

    assert_eq!(error, HttpError::CouldNotResolveHost);
}

#[test]
fn test_error_fail_to_connect() {
    let mut client = default_client();
    let request = default_get_request("http://localhost:9999".to_string());
    let error =  client.execute(&request, 0).err().unwrap();
    assert_eq!(error, HttpError::FailToConnect);


    let options = ClientOptions {
        follow_location: false,
        max_redirect: None,
        cookie_file: None,
        cookie_jar: None,
        proxy: Some("localhost:9999".to_string()),
        verbose: true,
    };
    let mut client = libcurl::client::Client::init(options);
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let error =  client.execute(&request, 0).err().unwrap();
    assert_eq!(error, HttpError::FailToConnect);

}


#[test]
fn test_error_could_not_resolve_proxy_name() {
    let options = ClientOptions {
        follow_location: false,
        max_redirect: None,
        cookie_file: None,
        cookie_jar: None,
        proxy: Some("unknown".to_string()),
        verbose: false,
    };
    let mut client = libcurl::client::Client::init(options);
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let error = client.execute(&request, 0).err().unwrap();
    assert_eq!(error, HttpError::CouldNotResolveProxyName);
}

// endregion

// region cookie

#[test]
fn test_cookie() {
    let mut client = default_client();
    let request = Request {
        method: Method::Get,
        url: "http://localhost:8000/cookies/set-request-cookie1-valueA".to_string(),
        headers: vec![],
        querystring: vec![],
        form: vec![],
        multipart: vec![],
        cookies: vec![
            RequestCookie { name: "cookie1".to_string(), value: "valueA".to_string() }
        ],
        body: vec![]
    };
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());


    // For the time-being setting a cookie on a request
    // update the cookie store as well
    // The same cookie does not need to be set explicitly on further requests
    let request = default_get_request("http://localhost:8000/cookies/set-request-cookie1-valueA".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

#[test]
fn test_cookie_storage() {
    let mut client = default_client();
    let request = default_get_request("http://localhost:8000/cookies/set-session-cookie2-valueA".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

    let cookie_store = client.get_cookie_storage();
    assert_eq!(cookie_store.get(0).unwrap().clone(), Cookie {
        domain: "localhost".to_string(),
        include_subdomain: "FALSE".to_string(),
        path: "/".to_string(),
        https: "FALSE".to_string(),
        expires: "0".to_string(),
        name: "cookie2".to_string(),
        value: "valueA".to_string(),
    });
    let request = default_get_request("http://localhost:8000/cookies/assert-that-cookie2-is-valueA".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}


#[test]
fn test_cookie_file() {
    let temp_file = "/tmp/cookies";
    let mut file = File::create(temp_file).expect("can not create temp file!");
    file.write_all(b"localhost\tFALSE\t/\tFALSE\t0\tcookie2\tvalueA\n").unwrap();

    let options = ClientOptions {
        follow_location: false,
        max_redirect: None,
        cookie_file: Some(temp_file.to_string()),
        cookie_jar: None,
        proxy: None,
        verbose: false,
    };
    let mut client = libcurl::client::Client::init(options);
    let request = default_get_request("http://localhost:8000/cookies/assert-that-cookie2-is-valueA".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());

}

// endregion

// region proxy

#[test]
fn test_proxy() {
    // mitmproxy listening on port 8080
    let options = ClientOptions {
        follow_location: false,
        max_redirect: None,
        cookie_file: None,
        cookie_jar: None,
        proxy: Some("localhost:8080".to_string()),
        verbose: false,
    };
    let mut client = libcurl::client::Client::init(options);
    let request = default_get_request("http://localhost:8000/hello".to_string());
    let response = client.execute(&request, 0).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"Hello World!".to_vec());
}

// endregion

