use foa::AppError;
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
enum CoreAppError {
    #[error("user with name \"{0}\" already exists")]
    UsernameDuplicate(String),

    #[error("username empty")]
    UsernameEmpty,
}

#[derive(Error, Debug, Serialize, Clone)]
#[error("error type 1")]
struct Err1;

#[derive(Error, Debug, Serialize, Clone)]
#[error("error type 2: foo={foo}, source=[{source}]")]
struct Err2 {
    foo: String,
    source: Err1,
}

#[derive(Error, Debug, Serialize, Clone)]
#[error("error type 3: bar={bar}, source=[{source}]")]
struct Err3 {
    bar: u32,
    source: Err2,
}

type AppErr = AppError<CoreAppError>;

fn main() {
    let core_err1 = AppErr::Core(CoreAppError::UsernameDuplicate("xyz".to_owned()));
    println!("{core_err1}");
    println!("{:?}", serde_json::to_string(&core_err1));
    println!("{}", serde_json::to_string(&core_err1).unwrap());
    println!("{core_err1:?}");

    println!();

    let core_err2 = AppErr::Core(CoreAppError::UsernameEmpty);
    println!("{core_err2}");
    println!("{:?}", serde_json::to_string(&core_err2));
    println!("{}", serde_json::to_string(&core_err2).unwrap());
    println!("{core_err2:?}");

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
        let lib_app_err = AppErr::library_error_ser(lib_err3.clone());
        println!("{lib_app_err}");
        println!("{:?}", serde_json::to_string(&lib_app_err));
        println!("{}", serde_json::to_string(&lib_app_err).unwrap());
        println!("{lib_app_err:?}");
    }

    println!();

    {
        let lib_app_err = AppErr::library_error(lib_err3);
        println!("{lib_app_err}");
        println!("{:?}", serde_json::to_string(&lib_app_err));
        println!("{}", serde_json::to_string(&lib_app_err).unwrap());
        println!("{lib_app_err:?}");
    }
}
