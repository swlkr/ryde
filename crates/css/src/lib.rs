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
        let src = stringify!(let class = css!(
            "color: var(--gray-300)",
            "justify-content: center",
            "display: flex"
        ); let class2 = css!("a: b"));
        let rules = rules(src);
        assert_eq!(
            "color:var(--gray-300),justify-content:center,display:flex,a:b",
            rules
        );
    }

    #[test]
    fn generate_works() {
        let src = stringify!(let class = css!(
            "color: var(--gray-300)",
            "justify-content: center",
            "display: flex",
        ););
        let expected = ".77dwKDE{color:var(--gray-300);}.VtriMut{justify-content:center;}.2tuJoSr{display:flex;}";
        let generated = generate(src);
        assert_eq!(expected, generated);
    }

    #[test]
    fn pseudo_variant_works() {
        let src = stringify!(let class = css!(
            "color: var(--gray-300)",
            "dark:hover:focus|color: var(--gray-300)",
        ););
        let expected = ".77dwKDE{color:var(--gray-300);}@media(prefers-color-scheme:dark){.FrFUmvk:focus:hover{color:var(--gray-300);}}";
        let generated = generate(src);
        assert_eq!(expected, generated);
    }
}
