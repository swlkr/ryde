pub use static_files_macros::StaticFiles;
extern crate self as static_files;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(StaticFiles, Debug, PartialEq)]
    #[folder("static")]
    struct StaticFile;

    #[test]
    fn it_works() {
        let uri = "/static/test.css";
        let (content_type, bytes) = StaticFile::get(uri).unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            "/* this is test.css */"
        );
        assert_eq!(content_type, "text/css");
    }
}
