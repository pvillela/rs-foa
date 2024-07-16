use foa::{ErrorKind, FoaError};

const ERROR1: ErrorKind =
    ErrorKind::new_with_args("ERROR1", "a dev message with '{}' as single arg", 1);

fn main() {
    println!("{ERROR1:?}");

    let err1 = FoaError::new_with_args(&ERROR1, [&42.to_string()]).unwrap();
    println!("{err1}");
    println!("{err1:?}");
}
