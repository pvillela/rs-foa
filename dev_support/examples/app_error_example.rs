#![allow(clippy::disallowed_names)]

use foa::error::AppError;
use serde::Serialize;
use std::fmt::Debug;

#[derive(thiserror::Error, Debug, Serialize)]
enum CoreAppError {
    #[error("user with name \"{0}\" already exists")]
    UsernameDuplicate(String),

    #[error("username empty")]
    UsernameEmpty,
}

#[derive(thiserror::Error, Debug, Serialize, Clone)]
#[error("error type 1")]
struct Err1;

#[derive(thiserror::Error, Debug, Serialize, Clone)]
#[error("error type 2: foo={foo}")]
struct Err2 {
    foo: String,
    source: Err1,
}

#[derive(thiserror::Error, Debug, Serialize, Clone)]
#[error("error type 3: bar={bar}")]
struct Err3 {
    bar: u32,
    source: Err2,
}

type AppErr = AppError<CoreAppError>;

fn main() {
    {
        let err = AppErr::Core(CoreAppError::UsernameDuplicate("xyz".to_owned()));
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("JSON option: {:?}", serde_json::to_string(&err));
        println!("JSON unwrapped: {}", serde_json::to_string(&err).unwrap());
    }
    println!();

    {
        let err = AppErr::Core(CoreAppError::UsernameEmpty);
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("JSON option: {:?}", serde_json::to_string(&err));
        println!("JSON unwrapped: {}", serde_json::to_string(&err).unwrap());
    }

    let lib_err1 = Err1;
    let lib_err2 = Err2 {
        foo: "abc".to_owned(),
        source: lib_err1,
    };
    let lib_err3 = Err3 {
        bar: 42,
        source: lib_err2,
    };

    println!();

    {
        let err = AppErr::library_error_ser(lib_err3.clone());
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("JSON option: {:?}", serde_json::to_string(&err));
        println!("JSON unwrapped: {}", serde_json::to_string(&err).unwrap());
    }

    println!();

    {
        let err = AppErr::library_error_std(lib_err3);
        println!("display: {err}");
        println!("debug: {err:?}");
        println!("JSON option: {:?}", serde_json::to_string(&err));
        println!("JSON unwrapped: {}", serde_json::to_string(&err).unwrap());
    }
}
