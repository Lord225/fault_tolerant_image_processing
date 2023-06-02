
// todo take it from main.rs to separate file
lazy_static::lazy_static!{
    pub static ref TEMP: String = std::env::var("IMG_TEMP").unwrap();
}

pub fn from_temp(path: &str) -> String {
    format!("{}/{}", *TEMP, path)
}