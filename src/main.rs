mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type MyError = Box<dyn std::error::Error>;
pub type MyResult<T> = std::result::Result<T, MyError>;

fn main() -> MyResult<()> {
    todo!()
}
