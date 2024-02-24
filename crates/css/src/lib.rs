extern crate self as css;

pub use css_macros::css;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_works() {
        let class = css!(
            "color: var(--gray-300)",
            "justify-content: center",
            "display: flex",
        );
        assert_eq!(
            "color-gray-300 justify-content-center display-flex",
            class.0
        );
        let class = css!("justify-content: start");
        assert_eq!("justify-content-start", class.0);
    }

    #[test]
    fn rules_works() {
        let css = css!(
            "color: var(--gray-300)",
            "justify-content: center",
            "display: flex"
        );
        assert_eq!("color-gray-300 justify-content-center display-flex", css.0);
        assert_eq!(
            vec![
                ".color-gray-300{color:var(--gray-300);}",
                ".justify-content-center{justify-content:center;}",
                ".display-flex{display:flex;}"
            ],
            css.1
        );
    }

    #[test]
    fn pseudo_variant_works() {
        let css = css!(
            "color: var(--gray-300)",
            "dark:hover:focus:color: var(--gray-300)",
        );
        let expected = vec![".color-gray-300{color:var(--gray-300);}", "@media(prefers-color-scheme:dark){.dark-hover-focus-color-gray-300:focus:hover{color:var(--gray-300);}}"];

        assert_eq!(expected, css.1);
    }
}
